//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
use crate::App;

mod first_boot;
pub use first_boot::run as run_first_boot;

pub fn init(app: &App) {
    first_boot::init(app);
}

pub fn tick(_app: &App) {}
