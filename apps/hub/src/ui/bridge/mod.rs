//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod countdowns;
mod inputs;
mod reminders;
mod voice;

pub use inputs::connect_inputs;
use std::sync::{Arc, Mutex};
pub use voice::spawn_bridge;

#[derive(Clone, Default)]
pub struct BridgeState {
    countdowns: Arc<Mutex<countdowns::Presentation>>,
}
