//! Fixture mỗi tính năng một file; ticket sau thêm `mod <feature>;` và gọi vào đây.
mod first_boot;
mod menu;
pub use first_boot::run as run_first_boot;
pub use menu::idle as menu_idle;
use crate::App;

pub mod home;
mod center;
pub use center::{add_timer, countdown_today, end_timer, focus_weather, stale};

pub fn init(app: &App) {
    super::settings::init(app);
    menu::init(app);
    home::init(app);
    first_boot::init(app);
    super::photos::init(app);
    center::init(app);
    super::reminders::init(app);
    super::timers::init(app);
}

pub fn tick(app: &App) {
    center::tick(app);
    super::timers::tick(app);
}
