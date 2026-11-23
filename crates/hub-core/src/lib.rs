//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod audio;
mod config;
mod conversation;
mod features;
mod ports;
mod runtime;
pub mod setup_web;
mod store;
mod ui_state;
mod voice_settings;

use std::sync::Arc;

pub use audio::{AudioCommand, AudioEvent, Sensitivity, Sound};
pub use config::{
    CalendarSettings, Config, ConfigStore, HomeSettings, VoiceSettings, WeatherSettings,
};
pub use ports::{Clock, SystemClock};
pub use runtime::run;
use tokio::sync::{mpsc, watch};
pub use ui_state::{UiState, VoiceSettingsState, VoiceState, VoiceUiState};
use voice_client::{VoiceCommand, VoiceConfig, VoiceEvent};
pub use voice_settings::DEFAULT_OTA_URL;

pub struct Ports {
    pub clock: Arc<dyn Clock>,
}

/// User inputs will be added with the corresponding behavior.
pub enum UserInput {
    SaveVoiceUrl(String),
    VoiceConnect,
    Setup(
        setup_web::Update,
        tokio::sync::oneshot::Sender<Result<(), String>>,
    ),
}

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>,
    pub voice_tx: mpsc::Sender<VoiceCommand>,
    pub voice_rx: mpsc::Receiver<VoiceEvent>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub audio_rx: mpsc::Receiver<AudioEvent>,
    pub input_rx: mpsc::Receiver<UserInput>,
    pub ui_tx: watch::Sender<UiState>,
}

pub use features::timers::Timer;
pub use store::Store;

pub fn tool_definitions() -> Vec<voice_client::ToolDef> {
    features::timers::tools::definitions()
}
