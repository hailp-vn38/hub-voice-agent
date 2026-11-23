mod audio;
pub(crate) mod effects;
pub(crate) mod jobs;
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
    let mut countdowns = crate::features::countdowns::Feature::load(&store)?;
    let mut weather = crate::features::weather::Feature::load(
        ports.weather.clone(),
        &config.get().weather,
        &store,
        ports.clock.now().timestamp().as_second(),
    )?;
    let (home_settings_tx, home_settings_rx) =
        tokio::sync::watch::channel(config.get().home.clone());
    let mut home_events = ports.home.subscribe(home_settings_rx);
    let mut home = crate::features::home::Feature::new(ports.home.clone());
    let mut conversation = Conversation::default();
    let mut alerts = crate::alerts::Alerts::default();
    let mut view = crate::view::View::new(ports.clock.now().timestamp().as_second());
    let mut jobs = jobs::Jobs::new();
    let mut tick = time::interval(Duration::from_secs(1));
    tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
    for effect in std::iter::once(weather.snapshot()).chain(
        timers
            .snapshot()
            .into_iter()
            .chain(countdowns.snapshot(ports.clock.now().date())),
    ) {
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
                pending.extend(countdowns.snapshot(ports.clock.now().date()));
                let old_screen = view.screen();
                pending.extend(view.tick(ports.clock.now().timestamp().as_second()));
                if old_screen == crate::Screen::Weather && view.screen() != crate::Screen::Weather { pending.extend(weather.leave(&store, ports.clock.now().timestamp().as_second())); }
                pending.extend(weather.tick(ports.clock.now().timestamp().as_second()));
                let now = ports.clock.now().timestamp().as_second();
                let due = timers.due(now);
                alerts.retain_due(&due);
                for timer in due { alerts.push(timer); }
                let (dismissed, effects) = alerts.tick(now, conversation.alert_blocked(), ports.system.time_synced());
                pending.extend(effects);
                if let Some(id) = dismissed { pending.extend(timers.dismiss(id)); }
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(event) = home_events.recv() => {
                for effect in home.event(event) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            event = jobs.next() => {
                match event {
                    jobs::Internal::Timeout(id, reply) => {
                        if id == "weather" { for effect in weather.on_job(Err(anyhow::anyhow!("Job timed out")), reply, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; } }
                        else if let Some(call) = reply { effects::execute(effects::Effect::ReplyError(call, "Job timed out"), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                    },
                    jobs::Internal::JobDone(id, result, reply) => {
                        if id == "weather" { for effect in weather.on_job(result, reply, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; } }
                        else if id.starts_with("home:") { if let Some(call) = reply { let response = result.map(|_| serde_json::json!({"ok": true})).map_err(|e| crate::features::countdowns::tools::error(&e.to_string())); effects::execute(effects::Effect::Reply(call, response), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; } }
                    },
                }
            },
            Some(event) = io.voice_rx.recv() => {
                match &event {
                    VoiceEvent::Connection(connection) => effects::execute_settings(effects::SettingsEffect::Connection(connection.clone()), &mut config, &io),
                    VoiceEvent::Activation { code } => effects::execute_settings(effects::SettingsEffect::Activation(code.clone()), &mut config, &io),
                    _ => {}
                }
                let old_screen = view.screen();
                let mut pending = match event {
                    VoiceEvent::ToolCall(call) => tools::route(&mut timers, &mut countdowns, &mut view, &mut weather, &home, &store, call, ports.clock.now()),
                    event => voice::on(event, &mut conversation, ports.clock.now()),
                };
                if old_screen == crate::Screen::Weather && view.screen() != crate::Screen::Weather { pending.extend(weather.leave(&store, ports.clock.now().timestamp().as_second())); }
                for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(event) = io.audio_rx.recv() => {
                if matches!(event, crate::AudioEvent::WakeWord) {
                    for effect in view.on_user(UserInput::Activity, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                }
                for effect in audio::on(event, &mut conversation, ports.clock.now()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
            }
            Some(input) = io.input_rx.recv() => match input {
                UserInput::DismissAlert => {
                    let (dismissed, effects) = alerts.dismiss();
                    let mut pending = effects;
                    if let Some(id) = dismissed { pending.extend(timers.dismiss(id)); }
                    for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                }
                UserInput::SaveVoiceUrl(input) => effects::execute_settings(effects::SettingsEffect::SaveUrl(input), &mut config, &io),
                UserInput::Setup(update, reply) => {
                    let weather_changed = matches!(&update, crate::setup_web::Update::Weather(_));
                    let home_changed = matches!(&update, crate::setup_web::Update::Home(_));
                    let result = config.update(|c| match update {
                        crate::setup_web::Update::Voice(url) => c.voice.ota_url = Some(url),
                        crate::setup_web::Update::Home(home) => c.home = home,
                        crate::setup_web::Update::Weather(weather) => c.weather = weather,
                        crate::setup_web::Update::Calendar(calendar) => c.calendar = calendar,
                    });
                    if result.is_ok() {
                        if home_changed { home_settings_tx.send_replace(config.get().home.clone()); }
                        if weather_changed { for effect in weather.update_home(&config.get().weather, &store, ports.clock.now().timestamp().as_second()) { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; } }
                        io.voice_config_tx.send_replace(crate::voice_settings::config(&config));
                        io.ui_tx.send_modify(|ui| ui.voice_settings.ota_url = config.get().voice.ota_url.clone().unwrap_or_default());
                    }
                    let _ = reply.send(result.map_err(|e| e.to_string()));
                }
                UserInput::VoiceConnect => { effects::execute(effects::Effect::Voice(VoiceCommand::Connect), &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                input => {
                    let old_screen = view.screen();
                    let mut pending = view.on_user(input, ports.clock.now().timestamp().as_second());
                    if old_screen == crate::Screen::Weather && view.screen() != crate::Screen::Weather { pending.extend(weather.leave(&store, ports.clock.now().timestamp().as_second())); }
                    for effect in pending { effects::execute(effect, &io.ui_tx, &io.voice_tx, &io.audio_tx, &mut jobs, &mut store).await?; }
                }
            },
            _ = io.ui_tx.closed() => return Ok(()),
        }
    }
}
