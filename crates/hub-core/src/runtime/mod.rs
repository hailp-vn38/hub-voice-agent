mod audio;
pub(crate) mod effects;
mod jobs;
mod tick;
mod voice;

use std::time::Duration;

use tokio::time;

use crate::{
    ConfigStore, Io, Ports, UserInput,
    conversation::{Conversation, Input},
    voice_settings,
};
use voice_client::{VoiceCommand, VoiceEvent};

pub async fn run(mut config: ConfigStore, ports: Ports, mut io: Io) -> anyhow::Result<()> {
    io.ui_tx
        .send_modify(|ui| ui.voice_settings = voice_settings::initial(&config));
    io.voice_config_tx
        .send_replace(voice_settings::config(&config));
    let mut conversation = Conversation::default();
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let mut pending = tick::tick(ports.clock.as_ref());
                pending.extend(conversation.on(Input::Tick, ports.clock.now()));
                for effect in pending {
                    effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await;
                }
            },
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(Some(call)) => effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await,
                    jobs::Internal::Timeout(None) => {},
                    jobs::Internal::JobDone(_id, _result, _reply) => {},
                }
            },
            Some(event) = io.voice_rx.recv() => {
                match &event {
                    VoiceEvent::Connection(connection) => io.ui_tx.send_modify(|ui| voice_settings::connection(&mut ui.voice_settings, connection.clone())),
                    VoiceEvent::Activation { code } => io.ui_tx.send_modify(|ui| ui.voice_settings.activation = Some(code.clone())),
                    _ => {}
                }
                let pending = voice::on(event, &mut conversation, ports.clock.now());
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await; }
            }
            Some(event) = io.audio_rx.recv() => {
                let pending = audio::on(event, &mut conversation, ports.clock.now());
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await; }
            }
            Some(input) = io.input_rx.recv() => match input {
                UserInput::SaveVoiceUrl(input) => {
                    let result = voice_settings::parse_url(&input).and_then(|url| {
                        let shown = url.to_string();
                        config.update(|c| c.voice.ota_url = Some(shown.clone()))?;
                        Ok(shown)
                    });
                    match result {
                        Ok(url) => {
                            io.voice_config_tx.send_replace(voice_settings::config(&config));
                            io.ui_tx.send_modify(|ui| { ui.voice_settings.ota_url = url; ui.voice_settings.error = None; });
                        }
                        Err(error) => io.ui_tx.send_modify(|ui| ui.voice_settings.error = Some(error.to_string())),
                    }
                }
                UserInput::VoiceConnect => { effects::execute(effects::Effect::Voice(VoiceCommand::Connect), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs).await; }
            },

            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
