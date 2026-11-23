//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod inputs;
mod voice;

pub use inputs::connect_inputs;
pub use voice::spawn_bridge;
