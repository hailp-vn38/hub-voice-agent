use super::WeatherBackground;
use crate::WeatherDay;
use jiff::{Span, Zoned};

pub(super) fn tomorrow(now: &Zoned) -> Option<String> {
    now.checked_add(Span::new().days(1))
        .ok()
        .map(|date| date.strftime("%F").to_string())
}

pub(super) fn day_label(date: &str, today: &str, tomorrow: Option<&str>) -> String {
    if date == today {
        return "Hôm nay".into();
    }
    if Some(date) == tomorrow {
        return "Mai".into();
    }
    date.parse::<jiff::civil::Date>()
        .map(|date| match date.weekday().to_monday_one_offset() {
            1 => "T2",
            2 => "T3",
            3 => "T4",
            4 => "T5",
            5 => "T6",
            6 => "T7",
            _ => "CN",
        })
        .unwrap_or(date)
        .to_owned()
}

pub(super) fn updated_time(now: &Zoned, updated: Option<i64>) -> String {
    let Some(updated) = updated else {
        return String::new();
    };
    now.checked_add(Span::new().seconds(updated.saturating_sub(now.timestamp().as_second())))
        .ok()
        .map(|time| time.strftime("%H:%M").to_string())
        .unwrap_or_default()
}

pub(in crate::present) fn is_day_at(now: &Zoned, day: Option<&WeatherDay>) -> bool {
    let (Some(sunrise), Some(sunset)) = (
        day.and_then(|day| day.sunrise.as_ref()),
        day.and_then(|day| day.sunset.as_ref()),
    ) else {
        return false;
    };
    let time = now.strftime("%FT%R").to_string();
    time.as_str() >= sunrise.as_str() && time.as_str() < sunset.as_str()
}

pub(super) fn background(code: Option<i64>) -> WeatherBackground {
    match code {
        Some(0) => WeatherBackground::Clear,
        Some(1..=3 | 71..=77 | 85..=86) => WeatherBackground::Cloudy,
        Some(45 | 48) => WeatherBackground::Fog,
        Some(51..=67 | 80..=82) => WeatherBackground::Rain,
        Some(95..=99) => WeatherBackground::Storm,
        _ => WeatherBackground::Cloudy,
    }
}

pub(in crate::present) fn icon(code: Option<i64>) -> &'static str {
    match code {
        Some(0) => "sun",
        Some(1..=3) => "cloud",
        Some(45 | 48) => "fog",
        Some(51..=67 | 80..=82) => "rain",
        Some(71..=77 | 85..=86) => "snow",
        Some(95..=99) => "storm",
        _ => "unknown",
    }
}

pub(super) fn summary(code: Option<i64>) -> &'static str {
    match code {
        Some(0) => "Trời quang",
        Some(1) => "Ít mây",
        Some(2) => "Có mây rải rác",
        Some(3) => "Nhiều mây",
        Some(45 | 48) => "Sương mù",
        Some(51..=57) => "Mưa phùn",
        Some(61..=67) => "Mưa",
        Some(71..=77 | 85..=86) => "Tuyết",
        Some(80..=82) => "Mưa rào",
        Some(95..=99) => "Giông",
        _ => "Không rõ",
    }
}

pub(super) fn aqi_view(aqi: f32) -> (String, String, String) {
    let (level, color) = match aqi as u16 {
        0..=50 => ("Tốt", "#00E400"),
        51..=100 => ("Trung bình", "#FFFF00"),
        101..=150 => ("Kém", "#FF7E00"),
        151..=200 => ("Xấu", "#FF0000"),
        201..=300 => ("Rất xấu", "#8F3F97"),
        _ => ("Nguy hại", "#7E0023"),
    };
    (format!("{aqi:.0}"), level.into(), color.into())
}

pub(super) fn temperature(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.0}°"))
        .unwrap_or_default()
}

pub(super) fn percent(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.0}%"))
        .unwrap_or_default()
}

pub(super) fn speed(value: Option<f32>) -> String {
    value
        .map(|value| format!("{value:.0} km/h"))
        .unwrap_or_default()
}

pub(super) fn number(value: Option<f32>) -> String {
    value.map(|value| format!("{value:.1}")).unwrap_or_default()
}

pub(super) fn clock(value: &Option<String>) -> Option<String> {
    value.as_deref()?.get(11..16).map(str::to_owned)
}
