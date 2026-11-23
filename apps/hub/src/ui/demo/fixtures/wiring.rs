//! Nạp fixture và tick 1 Hz cho mọi tính năng; ticket sau thêm một dòng vào đây.
use super::{calendar, center, countdowns, first_boot, home, menu, photos, reminders, settings, timers, weather};
use crate::App;

pub fn init(app: &App) {
    settings::init(app);
    menu::init(app);
    home::init(app);
    weather::init(app);
    first_boot::init(app);
    photos::init(app);
    center::init(app);
    calendar::init(app);
    reminders::init(app);
    timers::init(app);
    countdowns::init(app);
}

pub fn tick(app: &App) {
    center::tick(app);
    calendar::tick(app);
    timers::tick(app);
    countdowns::tick(app);
}
