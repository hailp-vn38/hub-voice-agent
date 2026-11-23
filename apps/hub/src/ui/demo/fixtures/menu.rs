//! Fixture dòng thông tin sống của menu.
use crate::{App, Menu};
use slint::ComponentHandle;

pub fn init(app: &App) {
    let m = app.global::<Menu>();
    m.set_weather("28°C, nắng nhẹ".into());
    m.set_calendar("3 sự kiện hôm nay".into());
    m.set_home("5 thiết bị đang bật".into());
    m.set_timers("1 đang chạy".into());
    m.set_countdowns("Tết còn 62 ngày".into());
    m.set_reminders("2 việc hôm nay".into());
    m.set_photos("128 ảnh".into());
    m.set_settings("Wi-Fi tốt".into());
}

/// Kịch bản "Không có gì đang chạy": mọi dòng về câu mặc định.
pub fn idle(app: &App) {
    let m = app.global::<Menu>();
    m.set_weather("Chưa có dữ liệu".into());
    m.set_calendar("Không có sự kiện".into());
    m.set_home("Không có thiết bị".into());
    m.set_timers("Không có gì đang chạy".into());
    m.set_countdowns("Chưa có đếm ngược".into());
    m.set_reminders("Không có việc".into());
    m.set_photos("Chưa có ảnh".into());
    m.set_settings("".into());
}
