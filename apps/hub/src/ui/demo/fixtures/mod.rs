//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
mod menu;
pub use menu::idle as menu_idle;
use crate::App;

pub mod home;

pub fn init(app: &App) {
    menu::init(app);
    home::init(app);
}

pub fn tick(_app: &App) {}
