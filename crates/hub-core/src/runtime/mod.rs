mod audio;
pub(crate) mod effects;
mod jobs;
mod tick;
mod tools;
mod voice;

use crate::{
    ConfigStore, Io, Ports, Store, UserInput,
    conversation::{Conversation, Input},
};
use std::time::Duration;
use tokio::time;
use voice_client::{VoiceCommand, VoiceEvent};

pub async fn run(
    mut config: ConfigStore,
    mut store: Store,
    ports: Ports,
    mut io: Io,
) -> anyhow::Result<()> {
    effects::execute_settings(effects::SettingsEffect::Initialize, &mut config, &io);
    let mut timers = crate::features::timers::Feature::load(&store)?;
    let mut conversation = Conversation::default();
    let mut view = crate::view::View::default();
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    for effect in timers.snapshot() {
        effects::execute(
            effect,
            &io.ui_tx,
            &io.voice_tx,
            &io.audio_tx,
            &mut jobs,
            &mut store,
        )
        .await?;
    }
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let mut pending = tick::tick(ports.clock.as_ref());
                pending.extend(conversation.on(Input::Tick, ports.clock.now()));
                pending.extend(view.tick(ports.clock.now().timestamp().as_second()));
                pending.extend(timers.tick(ports.clock.now().timestamp().as_second()));
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(Some(call)) => effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?,
                    jobs::Internal::Timeout(None) => {},
                    jobs::Internal::JobDone(_id, _result, _reply) => {},
                }
            },
            Some(event) = io.voice_rx.recv() => {
                match &event {
                    VoiceEvent::Connection(connection) => effects::execute_settings(effects::SettingsEffect::Connection(connection.clone()), &mut config, &io),
                    VoiceEvent::Activation { code } => effects::execute_settings(effects::SettingsEffect::Activation(code.clone()), &mut config, &io),
                    _ => {}
                }
                let pending = match event {
                    VoiceEvent::ToolCall(call) => tools::route(&mut timers, &mut view, call, ports.clock.now().timestamp().as_second()),
                    event => voice::on(event, &mut conversation, ports.clock.now()),
                };
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(event) = io.audio_rx.recv() => {
                if matches!(event, crate::AudioEvent::WakeWord) {
                    for effect in view.on_user(UserInput::Activity, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                }
                for effect in audio::on(event, &mut conversation, ports.clock.now()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(input) = io.input_rx.recv() => match input {
                UserInput::SaveVoiceUrl(input) => effects::execute_settings(effects::SettingsEffect::SaveUrl(input), &mut config, &io),
                UserInput::Setup(update, reply) => {
                    let result = config.update(|c| match update {
                        crate::setup_web::Update::Voice(url) => c.voice.ota_url = Some(url),
                        crate::setup_web::Update::Home(home) => c.home = home,
                        crate::setup_web::Update::Weather(weather) => c.weather = weather,
                        crate::setup_web::Update::Calendar(calendar) => c.calendar = calendar,
                    });
                    if result.is_ok() {
                        io.voice_config_tx.send_replace(crate::voice_settings::config(&config));
                        io.ui_tx.send_modify(|ui| ui.voice_settings.ota_url = config.get().voice.ota_url.clone().unwrap_or_default());
                    }
                    let _ = reply.send(result.map_err(|e| e.to_string()));
                }
                UserInput::VoiceConnect => { effects::execute(effects::Effect::Voice(VoiceCommand::Connect), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                input => for effect in view.on_user(input, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            },
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
