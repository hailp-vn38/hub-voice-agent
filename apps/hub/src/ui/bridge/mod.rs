//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod calendar;
mod inputs;
mod voice;
mod weather;

pub use inputs::connect_inputs;
pub use voice::spawn_bridge;
