use tokio::sync::{mpsc, watch};
use voice_client::{ToolCall, ToolError, VoiceCommand};

use super::jobs::{JobId, Jobs, Work};
use std::time::Duration;

use crate::{AudioCommand, UiState};

// Variants are introduced here for the subsequent voice and audio slices.
#[allow(dead_code)]
pub(super) enum Effect {
    Ui(Box<dyn FnOnce(&mut UiState) -> bool + Send>),
    Voice(VoiceCommand),
    Audio(AudioCommand),
    Spawn {
        job: JobId,
        work: Work,
        reply: Option<ToolCall>,
        timeout: Duration,
    },
    ReplyError(ToolCall, &'static str),
}

pub(super) async fn execute(
    effect: Effect,
    ui_tx: &watch::Sender<UiState>,
    voice_tx: &mpsc::Sender<VoiceCommand>,
    audio_tx: &mpsc::Sender<AudioCommand>,
    jobs: &mut Jobs,
) {
    match effect {
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
