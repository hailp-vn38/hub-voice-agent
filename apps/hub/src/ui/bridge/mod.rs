//! Adapter thật: UiState → global hợp đồng, callback hợp đồng → UserInput.
mod calendar;
mod center;
mod countdowns;
mod home;
mod images;
mod inputs;
mod location;
mod network;
mod photos;
mod reminders;
mod timers;
mod update;
mod voice;
mod weather;

pub use inputs::connect_inputs;
pub use location::connect_location;
pub use network::spawn_network;
pub use update::spawn_bridge;
