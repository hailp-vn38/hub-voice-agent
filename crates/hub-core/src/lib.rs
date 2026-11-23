//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod config;
mod ports;
mod runtime;
mod ui_state;
mod voice_settings;

use std::sync::Arc;

pub use config::{Config, ConfigStore, VoiceSettings};
pub use ports::{Clock, SystemClock};
pub use runtime::run;
use tokio::sync::{mpsc, watch};
pub use ui_state::{UiState, VoiceSettingsState};
use voice_client::{ToolDef, VoiceCommand, VoiceConfig, VoiceEvent};
pub use voice_settings::DEFAULT_OTA_URL;

pub fn tool_definitions() -> Vec<ToolDef> {
    vec![]
}

pub struct Ports {
    pub clock: Arc<dyn Clock>,
}

/// Audio commands will be added when audio behavior is implemented.
pub enum AudioCommand {}

/// User inputs will be added with the corresponding behavior.
pub enum UserInput {
    SaveVoiceUrl(String),
    VoiceConnect,
}

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>,
    pub voice_tx: mpsc::Sender<VoiceCommand>,
    pub voice_rx: mpsc::Receiver<VoiceEvent>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub input_rx: mpsc::Receiver<UserInput>,
    pub ui_tx: watch::Sender<UiState>,
}
