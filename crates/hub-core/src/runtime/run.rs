use super::{
    audio,
    display::Display,
    effects::{self, Effect, SettingsEffect, execute_all},
    hub::Hub,
    jobs::Jobs,
    setup,
};
use crate::{
    AudioCommand, AudioEvent, ConfigStore, Io, Ports, SettingChange, Sound, Store, UserInput,
};
use std::time::Duration;
use tokio::{sync::watch, time};
use voice_client::{VoiceCommand, VoiceEvent};

pub async fn run(
    mut config: ConfigStore,
    mut store: Store,
    ports: Ports,
    mut io: Io,
) -> anyhow::Result<()> {
    effects::execute_settings(SettingsEffect::Initialize, &mut config, &io);
    let mut hub = Hub::load(config.get(), &mut store, &ports)?;
    let mut display = Display::new(ports.device.clone());
    display.init(&config.get().device, &io);
    display.brightness(&config.get().device, &hub.now());
    let (home_settings, home_settings_rx) = watch::channel(config.get().home.clone());
    let mut home_events = ports.home.subscribe(home_settings_rx);
    let mut jobs = Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    let mut first = hub.snapshot();
    first.extend(audio::initial(&config.get().device));
    execute_all(first, &io, &mut jobs, &mut store).await?;
    loop {
        let effects = tokio::select! {
            _ = tick.tick() => {
                let now = hub.now();
                display.tick(now.timestamp().as_second(), &io);
                display.brightness(&config.get().device, &now);
                hub.tick(&store, config.get())
            }
            Some(event) = home_events.recv() => hub.on_home(event),
            event = jobs.next() => hub.on_job(event, &store),
            Some(event) = io.voice_rx.recv() => {
                match &event {
                    VoiceEvent::Connection(connection) => effects::execute_settings(
                        SettingsEffect::Connection(connection.clone()),
                        &mut config,
                        &io,
                    ),
                    VoiceEvent::Activation { code } => effects::execute_settings(
                        SettingsEffect::Activation(code.clone()),
                        &mut config,
                        &io,
                    ),
                    _ => {}
                }
                hub.on_voice(event, &store)
            }
            Some(event) = io.audio_rx.recv() => {
                if config.get().device.mic_muted
                    && matches!(event, AudioEvent::WakeWord | AudioEvent::MicFrame(_))
                {
                    Vec::new()
                } else {
                    hub.on_audio(event)
                }
            }
            Some(input) = io.input_rx.recv() => match input {
                UserInput::PinPhoto { id, pinned } => vec![Effect::PinPhoto(id, pinned)],
                UserInput::DismissAlert => hub.dismiss_alert(),
                UserInput::SaveVoiceUrl(input) => {
                    effects::execute_settings(SettingsEffect::SaveUrl(input), &mut config, &io);
                    Vec::new()
                }
                UserInput::Setting(SettingChange::Resolution(mode)) => {
                    display.try_resolution(mode, hub.now().timestamp().as_second(), &io);
                    Vec::new()
                }
                UserInput::PreviewSound(name) => Sound::preview(&name)
                    .map(|sound| Effect::Audio(AudioCommand::Play(sound)))
                    .into_iter()
                    .collect(),
                UserInput::ReminderSpeech(on) => {
                    effects::execute_settings(SettingsEffect::ReminderSpeech(on), &mut config, &io);
                    Vec::new()
                }
                UserInput::OrbTapped if config.get().device.mic_muted => Vec::new(),
                UserInput::KeepResolution => {
                    display.keep(&mut config, &io);
                    Vec::new()
                }
                UserInput::Setting(change) => {
                    effects::execute_settings(SettingsEffect::Apply(change), &mut config, &io);
                    display.brightness(&config.get().device, &hub.now());
                    audio::settings(&config.get().device)
                }
                UserInput::Setup(update, reply) => setup::apply(
                    update,
                    reply,
                    &mut config,
                    &mut hub,
                    &home_settings,
                    &io,
                    &store,
                ),
                UserInput::VoiceConnect => vec![Effect::Voice(VoiceCommand::Connect)],
                input => hub.on_input(input, &store),
            },
            _ = io.ui_tx.closed() => return Ok(()),
        };
        execute_all(effects, &io, &mut jobs, &mut store).await?;
    }
}
