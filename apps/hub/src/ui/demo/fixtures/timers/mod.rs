//! Demo Hẹn giờ (ticket 11): trạng thái mô phỏng, tự đếm ngược mỗi giây.
mod scenarios;
mod sim;
mod state;

pub use scenarios::*;
pub use state::{clear_all, init, tick};
