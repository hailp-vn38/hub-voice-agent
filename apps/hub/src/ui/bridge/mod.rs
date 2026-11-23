//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod calendar;
mod center;
mod countdowns;
mod home;
mod images;
mod inputs;
mod photos;
mod reminders;
mod timers;
mod voice;
mod weather;

pub use inputs::connect_inputs;
use std::sync::{Arc, Mutex};
pub use voice::spawn_bridge;

#[derive(Clone, Default)]
pub struct BridgeState {
    countdowns: Arc<Mutex<countdowns::Presentation>>,
}

#[cfg(test)]
#[path = "timers_tests.rs"]
mod timers_tests;

#[cfg(test)]
#[path = "center_tests.rs"]
mod center_tests;
