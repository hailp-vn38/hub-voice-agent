use tokio::sync::{mpsc, watch};
use voice_client::{ToolCall, ToolError, VoiceCommand};

use super::jobs::{JobId, Jobs, Work};
use std::time::Duration;

use crate::{AudioCommand, UiState};

// Variants are introduced here for the subsequent voice and audio slices.
#[allow(dead_code)]
pub(crate) enum Effect {
    Ui(Box<dyn FnOnce(&mut UiState) -> bool + Send>),
    Voice(VoiceCommand),
    Audio(AudioCommand),
    Spawn {
        job: JobId,
        work: Work,
        reply: Option<ToolCall>,
        timeout: Duration,
    },
    Persist(Vec<crate::Timer>),
    Reply(ToolCall, Result<serde_json::Value, ToolError>),
    ReplyError(ToolCall, &'static str),
}

pub(super) async fn execute(
    effect: Effect,
    ui_tx: &watch::Sender<UiState>,
    voice_tx: &mpsc::Sender<VoiceCommand>,
    audio_tx: &mpsc::Sender<AudioCommand>,
    jobs: &mut Jobs,
    store: &mut crate::Store,
) -> anyhow::Result<()> {
    match effect {
        Effect::Persist(timers) => store.timers_save(&timers)?,
        Effect::Reply(call, result) => call.reply(result),
        Effect::Ui(patch) => {
            ui_tx.send_if_modified(patch);
        }
        Effect::Voice(command) => {
            let _ = voice_tx.send(command).await;
        }
        Effect::Spawn {
            job,
            work,
            reply,
            timeout,
        } => {
            if let Some(call) = jobs.spawn(job, work, reply, timeout) {
                reply_error(call, "Job cancelled");
            }
        }
        Effect::ReplyError(call, message) => reply_error(call, message),
        Effect::Audio(command) => {
            let _ = audio_tx.send(command).await;
        }
    }
    Ok(())
}

fn reply_error(call: ToolCall, message: &str) {
    call.reply(Err(ToolError {
        code: -32603,
        message: message.into(),
        data: None,
    }));
}

#[cfg(test)]
#[path = "effects_tests.rs"]
mod tests;

/// Effects specific to persistent voice configuration.
pub(super) enum SettingsEffect {
    Initialize,
    SaveUrl(String),
    Connection(voice_client::ConnectionState),
    Activation(String),
}

pub(super) fn execute_settings(
    effect: SettingsEffect,
    config: &mut crate::ConfigStore,
    io: &crate::Io,
) {
    use crate::voice_settings;
    match effect {
        SettingsEffect::Initialize => {
            io.ui_tx
                .send_modify(|ui| ui.voice_settings = voice_settings::initial(config));
            io.voice_config_tx
                .send_replace(voice_settings::config(config));
        }
        SettingsEffect::SaveUrl(input) => {
            let result = voice_settings::parse_url(&input).and_then(|url| {
                let shown = url.to_string();
                config.update(|c| c.voice.ota_url = Some(shown.clone()))?;
                Ok(shown)
            });
            match result {
                Ok(url) => {
                    io.voice_config_tx
                        .send_replace(voice_settings::config(config));
                    io.ui_tx.send_modify(|ui| {
                        ui.voice_settings.ota_url = url;
                        ui.voice_settings.error = None;
                    });
                }
                Err(error) => io
                    .ui_tx
                    .send_modify(|ui| ui.voice_settings.error = Some(error.to_string())),
            }
        }
        SettingsEffect::Connection(connection) => io
            .ui_tx
            .send_modify(|ui| voice_settings::connection(&mut ui.voice_settings, connection)),
        SettingsEffect::Activation(code) => io
            .ui_tx
            .send_modify(|ui| ui.voice_settings.activation = Some(code)),
    }
}
