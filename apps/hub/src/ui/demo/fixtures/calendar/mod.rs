//! Fixture + kịch bản cho màn Lịch (ticket 10).
mod columns;
mod data;
mod state;
pub use state::{init, tick};
pub mod scenarios;

#[cfg(test)]
#[path = "columns_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "data_tests.rs"]
mod data_tests;
