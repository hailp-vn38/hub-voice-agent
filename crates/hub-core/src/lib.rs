//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod ports;
mod runtime;
mod ui_state;

use std::sync::Arc;

pub use ports::{Clock, SystemClock};
use tokio::sync::watch;
pub use ui_state::UiState;
pub use runtime::run;

pub struct Ports {
    pub clock: Arc<dyn Clock>,
}

pub struct Io {
    pub ui_tx: watch::Sender<UiState>,
}

