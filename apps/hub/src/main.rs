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
    app.set_ota_url(
        settings
            .ota_url()
            .map_or_else(String::new, |url| url.to_string())
            .into(),
    );
    app.set_device_id(settings.device_id().into());
    app.set_voice_status("Chưa cấu hình".into());
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
            tokio::select! {
                result = hub_core::run(ports, Io { ui_tx }) => result,
                _ = stop_rx => Ok(()),
            }
        })
    });

    let commands = settings_rx.recv()?;
    let save_commands = commands.clone();
    app.on_save_voice_url(move |url| {
        let _ = save_commands.send(SettingsCommand::SaveUrl(url.to_string()));
    });
    app.on_connect_voice(move || {
        let _ = commands.send(SettingsCommand::Connect);
    });

    app.run()?;
    let _ = stop_tx.send(());
    core.join()
        .map_err(|_| anyhow::anyhow!("hub-core thread panicked"))?
}
