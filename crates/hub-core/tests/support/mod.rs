#[allow(dead_code)]
mod weather;
#[allow(unused_imports)]
pub use weather::FakeWeather;

use hub_core::Clock;
use jiff::{Span, Zoned};
use tokio::time::Instant;

/// Clock that starts at a fixed instant and advances with tokio's (paused) time.
pub struct TestClock {
    start: Zoned,
    origin: Instant,
}

impl TestClock {
    pub fn at(start: Zoned) -> Self {
        Self {
            start,
            origin: Instant::now(),
        }
    }
}

impl Clock for TestClock {
    fn now(&self) -> Zoned {
        let ms = self.origin.elapsed().as_millis() as i64;
        self.start
            .checked_add(Span::new().milliseconds(ms))
            .unwrap()
    }
}

use std::sync::Arc;

use hub_core::{AudioCommand, AudioEvent, Io, Ports, UiState, UserInput, run};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use voice_client::{VoiceCommand, VoiceEvent};

#[allow(dead_code)]
pub struct Hub {
    pub voice_rx: mpsc::Receiver<VoiceCommand>,
    pub voice_tx: mpsc::Sender<VoiceEvent>,
    pub audio_rx: mpsc::Receiver<AudioCommand>,
    pub audio_tx: mpsc::Sender<AudioEvent>,
    pub input_tx: mpsc::Sender<UserInput>,
    pub ui_rx: watch::Receiver<UiState>,
    pub config_rx: watch::Receiver<Option<voice_client::VoiceConfig>>,
    pub _dir: tempfile::TempDir,
    pub task: JoinHandle<anyhow::Result<()>>,
}

impl Hub {
    pub fn start(start: Zoned) -> Self {
        Self::start_with_store(start, hub_core::Store::in_memory().unwrap())
    }

    pub fn start_with_store(start: Zoned, store: hub_core::Store) -> Self {
        let (voice_tx, voice_rx) = mpsc::channel(16);
        let (voice_event_tx, voice_event_rx) = mpsc::channel(16);
        let (audio_tx, audio_rx) = mpsc::channel(16);
        let (audio_event_tx, audio_event_rx) = mpsc::channel(16);
        let (input_tx, input_rx) = mpsc::channel(16);
        let (ui_tx, ui_rx) = watch::channel(UiState::default());
        let dir = tempfile::tempdir().unwrap();
        let config = hub_core::ConfigStore::open(dir.path()).unwrap();
        let (voice_config_tx, config_rx) = watch::channel(None);
        let task = tokio::spawn(run(
            config,
            store,
            Ports {
                clock: Arc::new(TestClock::at(start)),
                weather: Arc::new(hub_core::OpenMeteo::default()),
                home: Arc::new(hub_core::HomeAssistant::default()),
                system: Arc::new(Synced),
            },
            Io {
                voice_config_tx,
                voice_tx,
                voice_rx: voice_event_rx,
                audio_tx,
                audio_rx: audio_event_rx,
                input_rx,
                ui_tx,
            },
        ));
        Self {
            voice_rx,
            voice_tx: voice_event_tx,
            audio_rx,
            audio_tx: audio_event_tx,
            input_tx,
            ui_rx,
            config_rx,
            _dir: dir,
            task,
        }
    }
}

pub struct Synced;
impl hub_core::System for Synced {
    fn time_synced(&self) -> bool {
        true
    }
}
