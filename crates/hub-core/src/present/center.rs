use super::{
    CountdownUnit, CountdownsView, HomeTileView, HomeView, RemindersView, TimersView,
    WeatherDataView, calendar,
};
use crate::{Focus, UiState, lunar::to_lunar};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CenterCard {
    #[default]
    None,
    Weather,
    Calendar,
    Reminders,
    Countdown,
    Home,
    Timer,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CenterWeather {
    pub icon: String,
    pub temp: String,
    pub summary: String,
    pub updated: String,
    pub stale: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CenterEvent {
    pub key: String,
    pub time: String,
    pub title: String,
    pub now: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CenterReminder {
    pub key: String,
    pub time: String,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CenterTimer {
    pub visible: bool,
    pub key: String,
    pub label: String,
    pub remaining: String,
    pub progress: f32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CenterCountdown {
    pub visible: bool,
    pub key: String,
    pub label: String,
    pub value: String,
    pub today: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CenterView {
    pub time: String,
    pub date: String,
    pub weather: CenterWeather,
    pub today: Vec<CenterEvent>,
    pub reminders: Vec<CenterReminder>,
    pub timer: CenterTimer,
    pub countdown: CenterCountdown,
    pub home: Vec<HomeTileView>,
    pub home_safe: bool,
    pub focus: CenterCard,
    pub focus_serial: u64,
}

pub(super) fn from_state(
    state: &UiState,
    countdowns: &CountdownsView,
    home: &HomeView,
    reminders: &RemindersView,
    timers: &TimersView,
    weather: &WeatherDataView,
) -> CenterView {
    CenterView {
        time: state.now.strftime("%H:%M").to_string(),
        date: date_label(state),
        weather: center_weather(weather),
        today: calendar::center_today(state),
        reminders: reminders
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .map(|reminder| CenterReminder {
                key: reminder.key.clone(),
                time: reminder.time.clone(),
                text: reminder.text.clone(),
            })
            .collect(),
        timer: center_timer(state, timers),
        countdown: center_countdown(countdowns),
        home: home.tiles.iter().take(4).cloned().collect(),
        home_safe: state.home_connected,
        focus: center_focus(state),
        focus_serial: state.events.focus,
    }
}

fn date_label(state: &UiState) -> String {
    let date = state.now.date();
    let weekday = match date.weekday().to_monday_one_offset() {
        1 => "Thứ Hai",
        2 => "Thứ Ba",
        3 => "Thứ Tư",
        4 => "Thứ Năm",
        5 => "Thứ Sáu",
        6 => "Thứ Bảy",
        _ => "Chủ nhật",
    };
    let lunar = to_lunar(date);
    let leap = if lunar.leap { " nhuận" } else { "" };
    format!(
        "{weekday}, {:02}/{:02} · {}/{}{} âm lịch",
        date.day(),
        date.month(),
        lunar.day,
        lunar.month,
        leap
    )
}

fn center_weather(weather: &WeatherDataView) -> CenterWeather {
    if weather.now.temp.is_empty()
        && (weather.now.icon.is_empty() || weather.now.summary == "Không rõ")
    {
        return CenterWeather::default();
    }
    CenterWeather {
        icon: weather.now.icon.clone(),
        temp: weather.now.temp.clone(),
        summary: weather.now.summary.clone(),
        updated: weather.updated.clone(),
        stale: weather.stale,
    }
}

fn center_timer(state: &UiState, timers: &TimersView) -> CenterTimer {
    let now = state.now.timestamp().as_second();
    let nearest = state
        .timers
        .iter()
        .filter(|timer| timer.paused_seconds.is_none() && timer.end_seconds > now)
        .min_by_key(|timer| timer.end_seconds);
    let Some(nearest) = nearest else {
        return CenterTimer::default();
    };
    let Some(card) = timers
        .items
        .iter()
        .find(|card| card.key == nearest.id.to_string())
    else {
        return CenterTimer::default();
    };
    CenterTimer {
        visible: true,
        key: card.key.clone(),
        label: card.label.clone(),
        remaining: card.remaining.clone(),
        progress: card.progress,
    }
}

fn center_countdown(countdowns: &CountdownsView) -> CenterCountdown {
    if !countdowns.has_main {
        return CenterCountdown::default();
    }
    CenterCountdown {
        visible: true,
        key: countdowns.main.key.clone(),
        label: countdowns.main.label.clone(),
        value: countdowns.main.value.clone(),
        today: countdowns.main.unit == CountdownUnit::Today,
    }
}

fn center_focus(state: &UiState) -> CenterCard {
    match state.view.focus.as_ref() {
        Some(Focus::Weather(_)) => CenterCard::Weather,
        Some(Focus::Calendar(_)) => CenterCard::Calendar,
        Some(Focus::Countdown(_)) => CenterCard::Countdown,
        Some(Focus::Timer(_)) => CenterCard::Timer,
        Some(Focus::Photo(_)) | None => CenterCard::None,
    }
}
