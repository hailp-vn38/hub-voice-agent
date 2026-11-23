//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod audio;
mod conversation;
mod features;
pub mod lunar;
pub mod selector;

mod ports;
mod runtime;
mod store;
mod ui_state;

use std::sync::Arc;

pub use audio::{AudioCommand, AudioEvent, Sensitivity, Sound};
pub use ports::{Clock, SystemClock};
pub use runtime::run;
use tokio::sync::{mpsc, watch};
pub use ui_state::{UiState, VoiceState, VoiceUiState};
use voice_client::{VoiceCommand, VoiceEvent};

pub struct Ports {
    pub clock: Arc<dyn Clock>,
}

/// User inputs will be added with the corresponding behavior.
pub enum UserInput {}

pub struct Io {
    pub voice_tx: mpsc::Sender<VoiceCommand>,
    pub voice_rx: mpsc::Receiver<VoiceEvent>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub audio_rx: mpsc::Receiver<AudioEvent>,
    pub input_rx: mpsc::Receiver<UserInput>,
    pub ui_tx: watch::Sender<UiState>,
}

pub use features::countdowns::{Countdown, CountdownView};
pub use features::timers::Timer;
pub use store::Store;

pub fn tool_definitions() -> Vec<voice_client::ToolDef> {
    let mut tools = features::timers::tools::definitions();
    tools.extend(features::countdowns::tools::definitions());
    tools
}
