#[cfg(not(feature = "demo"))]
mod bridge;
#[cfg(feature = "demo")]
pub mod demo;
#[cfg(not(feature = "demo"))]
pub use bridge::{BridgeState, connect_inputs, spawn_bridge};
