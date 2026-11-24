#[cfg(not(feature = "demo"))]
mod bridge;
#[cfg(feature = "demo")]
pub mod demo;
#[cfg(not(feature = "demo"))]
pub use bridge::{connect_inputs, spawn_bridge};
