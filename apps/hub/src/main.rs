#[cfg(all(feature = "desktop", feature = "pi"))]
compile_error!(
    "features `desktop` and `pi` are mutually exclusive; for the Pi use `--no-default-features --features pi`"
);
#[cfg(not(any(feature = "desktop", feature = "pi")))]
compile_error!("enable exactly one backend feature: `desktop` (default) or `pi`");

mod ui;
use hub_core::{ConfigStore, Io, Ports, SystemClock, UiState, UserInput};
use slint::ComponentHandle;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::{mpsc, oneshot, watch};
slint::include_modules!();

#[cfg(feature = "demo")]
fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let _demo = ui::demo::start(&app)?;
    app.run()?;
    Ok(())
}

#[cfg(not(feature = "demo"))]
fn main() -> anyhow::Result<()> {
    let app = App::new()?;
    let data_dir = std::env::var_os("AI_HUB_DATA_DIR")
        .or_else(|| std::env::var_os("STATE_DIRECTORY"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".data"));
    let config = ConfigStore::open(&data_dir)?;
    let store = hub_core::Store::open(&data_dir)?;
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let (config_tx, config_rx) = watch::channel(None);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let weak = app.as_weak();
    let core = thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        rt.block_on(async move {
            let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
            ui::spawn_bridge(weak, ui_rx);
            let ports = Ports { clock: Arc::new(SystemClock) };
            let (audio_tx, mut audio_commands) = mpsc::channel(16);
            let (_audio_events, audio_rx) = mpsc::channel(16);
            tokio::spawn(async move { while audio_commands.recv().await.is_some() {} });
            tokio::select! {
                result = hub_core::run(config, store, ports, Io { voice_config_tx: config_tx, voice_tx, voice_rx, audio_tx, audio_rx, input_rx, ui_tx }) => result,
                _ = stop_rx => Ok(()),
            }
        })
    });
    let save_tx = input_tx.clone();
    app.on_save_voice_url(move |url| {
        if save_tx
            .try_send(UserInput::SaveVoiceUrl(url.to_string()))
            .is_err()
        {
            eprintln!("hub-core không nhận được URL");
        }
    });
    app.on_connect_voice(move || {
        if input_tx.try_send(UserInput::VoiceConnect).is_err() {
            eprintln!("hub-core không nhận được lệnh kết nối");
        }
    });
    app.run()?;
    let _ = stop_tx.send(());
    core.join()
        .map_err(|_| anyhow::anyhow!("hub-core thread panicked"))?
}
