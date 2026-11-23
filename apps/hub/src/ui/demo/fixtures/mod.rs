//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
mod first_boot;
mod menu;
pub use first_boot::run as run_first_boot;
pub use menu::idle as menu_idle;
use crate::App;

pub mod home;

pub fn init(app: &App) {
    super::settings::init(app);
    menu::init(app);
    home::init(app);
    first_boot::init(app);
    super::photos::init(app);
}

pub fn tick(_app: &App) {}
