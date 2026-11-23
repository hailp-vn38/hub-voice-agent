use super::{CenterView, CountdownsView, HomeView, PhotosView, RemindersView};
use crate::{Timer, UiState};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuView {
    pub weather: String,
    pub calendar: String,
    pub home: String,
    pub timers: String,
    pub countdowns: String,
    pub reminders: String,
    pub photos: String,
    pub settings: String,
}

pub(super) fn from_state(
    state: &UiState,
    center: &CenterView,
    countdowns: &CountdownsView,
    home: &HomeView,
    photos: &PhotosView,
    reminders: &RemindersView,
) -> MenuView {
    MenuView {
        weather: weather_line(center),
        calendar: calendar_line(center.today.len()),
        home: home_line(state, home),
        timers: timers_line(&state.timers, state.now.timestamp().as_second()),
        countdowns: countdowns_line(state, countdowns),
        reminders: reminders_line(reminders),
        photos: photos_line(photos.photos.len()),
        settings: if state.voice_settings.status_label.is_empty() {
            "Chưa cấu hình".into()
        } else {
            state.voice_settings.status_label.clone()
        },
    }
}

fn weather_line(center: &CenterView) -> String {
    let weather = &center.weather;
    if weather.temp.is_empty() && weather.summary.is_empty() {
        return "Chưa có dữ liệu".into();
    }
    let mut parts = Vec::new();
    if !weather.temp.is_empty() {
        parts.push(weather.temp.as_str());
    }
    if !weather.summary.is_empty() {
        parts.push(weather.summary.as_str());
    }
    let stale = if weather.stale { " · cũ" } else { "" };
    format!("{}{stale}", parts.join(" · "))
}

fn calendar_line(count: usize) -> String {
    if count == 0 {
        "Không có sự kiện hôm nay".into()
    } else {
        format!("{count} sự kiện hôm nay")
    }
}

fn home_line(state: &UiState, home: &HomeView) -> String {
    if !home.configured {
        return "Chưa cấu hình".into();
    }
    if !home.connected {
        return "Mất kết nối".into();
    }
    if state.home.is_empty() {
        return "Không có thiết bị".into();
    }
    format!("{} thiết bị", state.home.len())
}

fn timers_line(timers: &[Timer], now: i64) -> String {
    let count = timers
        .iter()
        .filter(|timer| timer.paused_seconds.is_none() && timer.end_seconds > now)
        .count();
    if count == 0 {
        "Không có hẹn giờ đang chạy".into()
    } else {
        format!("{count} hẹn giờ đang chạy")
    }
}

fn countdowns_line(state: &UiState, countdowns: &CountdownsView) -> String {
    let count = state
        .countdowns
        .iter()
        .filter(|countdown| countdown.days >= 0)
        .count();
    if !countdowns.has_main || count == 0 {
        "Không có mốc sắp tới".into()
    } else {
        format!("{count} mốc sắp tới")
    }
}

fn reminders_line(reminders: &RemindersView) -> String {
    let count = reminders
        .groups
        .iter()
        .map(|group| group.items.len())
        .sum::<usize>();
    if count == 0 {
        "Không có lời nhắc sắp tới".into()
    } else {
        format!("{count} lời nhắc sắp tới")
    }
}

fn photos_line(count: usize) -> String {
    if count == 0 {
        "Chưa có ảnh".into()
    } else {
        format!("{count} ảnh")
    }
}
