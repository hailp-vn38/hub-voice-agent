//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod home;
mod images;
mod inputs;
mod photos;
mod timers;
mod voice;

pub use inputs::connect_inputs;
pub use voice::spawn_bridge;

#[cfg(test)]
#[path = "timers_tests.rs"]
mod timers_tests;
