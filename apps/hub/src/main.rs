#[cfg(all(feature = "desktop", feature = "pi"))]
compile_error!(
    "features `desktop` and `pi` are mutually exclusive; for the Pi use `--no-default-features --features pi`"
);

#[cfg(not(any(feature = "desktop", feature = "pi")))]
compile_error!("enable exactly one backend feature: `desktop` (default) or `pi`");

mod ui;

use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    thread,
};

use hub::voice_settings::{SettingsCommand, VoiceSettings};
use hub_core::{Io, Ports, SystemClock, UiState};
use slint::ComponentHandle;
use tokio::sync::{oneshot, watch};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let data_dir = std::env::var_os("AI_HUB_DATA_DIR")
        .or_else(|| std::env::var_os("STATE_DIRECTORY"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".data"));
    let settings = VoiceSettings::open(&data_dir)?;
    let store = hub_core::Store::open(&data_dir)?;
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let (settings_tx, settings_rx) = mpsc::sync_channel(1);

    let weak = app.as_weak();
    let core = thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move {
            let (commands, voice_state) = hub::voice_settings::start(settings);
            settings_tx.send(commands)?;
            ui::spawn_voice_bridge(weak.clone(), voice_state);
            ui::spawn_bridge(weak, ui_rx);
            let ports = Ports {
                clock: Arc::new(SystemClock),
            };
            let (voice_tx, _voice_commands) = tokio::sync::mpsc::channel(16);
            let (_voice_events, voice_rx) = tokio::sync::mpsc::channel(16);
            let (audio_tx, mut _audio_commands) = tokio::sync::mpsc::channel(16);
            let (_audio_events, audio_rx) = tokio::sync::mpsc::channel(16);
            tokio::spawn(async move { while _audio_commands.recv().await.is_some() {} });
            let (_inputs, input_rx) = tokio::sync::mpsc::channel(16);
            tokio::select! {
                result = hub_core::run(store, ports, Io { voice_tx, voice_rx, audio_tx, audio_rx, input_rx, ui_tx }) => result,
                _ = stop_rx => Ok(()),
            }
        })
    });

    let commands = settings_rx.recv()?;
    let save_commands = commands.clone();
    app.on_save_voice_url(move |url| {
        if save_commands
            .send(SettingsCommand::SaveUrl(url.to_string()))
            .is_err()
        {
            eprintln!("voice settings controller stopped; URL not saved");
        }
    });
    app.on_connect_voice(move || {
        if commands.send(SettingsCommand::Connect).is_err() {
            eprintln!("voice settings controller stopped; Connect ignored");
        }
    });

    app.run()?;
    let _ = stop_tx.send(());
    core.join()
        .map_err(|_| anyhow::anyhow!("hub-core thread panicked"))?
}
