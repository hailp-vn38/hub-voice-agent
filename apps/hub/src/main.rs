#[cfg(all(feature = "desktop", feature = "kiosk"))]
compile_error!(
    "features `desktop` and `kiosk` are mutually exclusive; for the tablet use `--no-default-features --features kiosk`"
);
#[cfg(not(any(feature = "desktop", feature = "kiosk")))]
compile_error!("enable exactly one backend feature: `desktop` (default) or `kiosk`");

mod audio;
mod device;
mod ui;
use hub_core::{ConfigStore, Io, Ports, SystemClock, UiState};
use slint::ComponentHandle;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::{mpsc, oneshot, watch};
slint::include_modules!();

/// Cổng của Setup web; Settings hiển thị `http://<ip LAN>:SETUP_PORT`.
const SETUP_PORT: u16 = 8080;

struct HostSystem;
impl hub_core::System for HostSystem {
    /// The desktop clock is trusted. On the kiosk tablet, systemd-timesyncd creates this
    /// file once NTP has synchronized the clock.
    fn time_synced(&self) -> bool {
        cfg!(feature = "desktop")
            || std::path::Path::new("/run/systemd/timesync/synchronized").exists()
    }
}

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
    let setup_tx = input_tx.clone();
    let location_tx = input_tx.clone();
    let (handle_tx, handle_rx) = std::sync::mpsc::channel();
    let weak = app.as_weak();
    let core = thread::spawn(move || -> anyhow::Result<()> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let _ = handle_tx.send(rt.handle().clone());
        rt.block_on(async move {
            let listener = tokio::net::TcpListener::bind(("0.0.0.0", SETUP_PORT)).await?;
            tokio::spawn(async move {
                if let Err(error) = hub_core::setup_web::serve(listener, setup_tx).await {
                    eprintln!("Setup web dừng: {error}");
                }
            });
            let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
            let audio = audio::spawn().unwrap_or_else(|error| {
                eprintln!("Chạy không có âm thanh: {error:#}");
                audio::disabled()
            });
            let audio_tx = audio.commands;
            let audio_rx = audio.events;
            ui::spawn_network(weak.clone(), SETUP_PORT);
            ui::spawn_bridge(weak, ui_rx, audio.levels);
            let ports = Ports { clock: Arc::new(SystemClock), system: Arc::new(HostSystem), device: Arc::new(device::HostDevice::new()), calendar: Arc::new(hub_core::GoogleCalendar::default()), weather: Arc::new(hub_core::OpenMeteo::default()), home: Arc::new(hub_core::HomeAssistant::default()), images: Arc::new(hub_core::HttpImageFetcher) };
            tokio::select! {
                result = hub_core::run(config, store, ports, Io { voice_config_tx: config_tx, voice_tx, voice_rx, audio_tx, audio_rx, input_rx, ui_tx }) => result,
                _ = stop_rx => Ok(()),
            }
        })
    });
    ui::connect_inputs(&app, input_tx);
    if let Ok(handle) = handle_rx.recv() {
        ui::connect_location(&app, location_tx, handle);
    }
    app.run()?;
    let _ = stop_tx.send(());
    core.join()
        .map_err(|_| anyhow::anyhow!("hub-core thread panicked"))?
}
