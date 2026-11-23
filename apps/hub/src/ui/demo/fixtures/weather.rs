//! Fixture Thời tiết: Hà Nội (nhà) và Đà Nẵng (tạm xem).
use crate::{App, WeatherBg, WeatherData, WeatherDay, WeatherHour, WeatherNow};
use slint::{Color, ComponentHandle, ModelRc, SharedString, VecModel};
use std::rc::Rc;

const DAY_LABELS: [&str; 7] = ["Hôm nay", "Mai", "T6", "T7", "CN", "T2", "T3"];
const DAY_KEYS: [&str; 7] = ["d0", "d1", "d2", "d3", "d4", "d5", "d6"];

/// Nhiệt độ 24 giờ: thấp nhất 5h, cao nhất 14h, đúng bằng low/high.
fn hour_temps(low: f32, high: f32) -> Vec<f32> {
    (0..24)
        .map(|h| {
            let x = (h as f32 - 14.0) * std::f32::consts::PI / 12.0; // cos = 1 tại 14h
            let t = low + (high - low) * (x.cos() + 1.0) / 2.0;
            if h == 14 { high } else if h == 2 { low } else { t.round() }
        })
        .collect()
}

/// Khả năng mưa theo giờ (0..1), đỉnh buổi chiều, nhân với `chance`.
fn hour_rain(chance: f32) -> Vec<f32> {
    (0..24).map(|h| (chance * (1.0 - ((h as f32 - 16.0) / 12.0).abs()).max(0.05)).clamp(0.0, 1.0)).collect()
}

fn week_range(days: &[(f32, f32)]) -> (f32, f32) {
    days.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &(l, h)| (lo.min(l), hi.max(h)))
}

struct Place {
    name: &'static str,
    days: [(f32, f32); 7],
    rain: [f32; 7],
}

const HANOI: Place = Place {
    name: "Hà Nội",
    days: [(24.0, 31.0), (23.0, 28.0), (22.0, 29.0), (24.0, 33.0), (25.0, 35.0), (23.0, 30.0), (21.0, 27.0)],
    rain: [0.1, 0.8, 0.3, 0.1, 0.0, 0.5, 0.6],
};
const DANANG: Place = Place {
    name: "Đà Nẵng",
    days: [(26.0, 33.0), (26.0, 32.0), (25.0, 31.0), (25.0, 30.0), (26.0, 34.0), (27.0, 35.0), (26.0, 33.0)],
    rain: [0.0, 0.2, 0.4, 0.6, 0.1, 0.0, 0.1],
};

fn show_day(app: &App, p: &Place, i: usize) {
    let w = app.global::<WeatherData>();
    let (low, high) = p.days[i];
    let rain = hour_rain(p.rain[i]);
    let hours: Vec<WeatherHour> = hour_temps(low, high)
        .into_iter()
        .enumerate()
        .map(|(h, t)| WeatherHour {
            key: format!("{}-h{h}", DAY_KEYS[i]).into(),
            label: format!("{h:02}:00").into(),
            temp: t,
            rain: rain[h],
            wind: format!("{} km/h", 8 + h % 5).into(),
            icon: "cloud".into(),
        })
        .collect();
    w.set_hours(ModelRc::from(Rc::new(VecModel::from(hours))));
    w.set_selected_day(DAY_KEYS[i].into());
}

fn load(app: &App, p: &'static Place) {
    let w = app.global::<WeatherData>();
    w.set_location(p.name.into());
    let (lo, hi) = week_range(&p.days);
    w.set_week_min(lo);
    w.set_week_max(hi);
    let days: Vec<WeatherDay> = (0..7)
        .map(|i| WeatherDay {
            key: DAY_KEYS[i].into(),
            label: DAY_LABELS[i].into(),
            icon: if p.rain[i] > 0.4 { "cloud" } else { "sun" }.into(),
            low: p.days[i].0,
            high: p.days[i].1,
            low_text: format!("{}°", p.days[i].0).into(),
            high_text: format!("{}°", p.days[i].1).into(),
            is_tomorrow: i == 1,
        })
        .collect();
    w.set_days(ModelRc::from(Rc::new(VecModel::from(days))));
    w.set_now(WeatherNow {
        icon: "sun".into(),
        temp: format!("{}°", p.days[0].1 - 3.0).into(),
        summary: "Nắng nhẹ, vài đám mây".into(),
        feels: format!("{}°", p.days[0].1).into(),
        humidity: "72%".into(),
        wind: "12 km/h".into(),
        uv: "7 (cao)".into(),
        aqi: "82".into(),
        aqi_level: "Trung bình".into(),
        aqi_color: Color::from_rgb_u8(0xff, 0xd1, 0x4a),
        sunrise: "05:42".into(),
        sunset: "18:21".into(),
    });
    let weak = app.as_weak();
    w.on_select_day(move |k| {
        if let Some(app) = weak.upgrade() {
            if let Some(i) = DAY_KEYS.iter().position(|d| *d == k.as_str()) {
                show_day(&app, p, i);
            }
        }
    });
    show_day(app, p, 0);
}

pub fn init(app: &App) {
    let w = app.global::<WeatherData>();
    w.set_updated("12:00".into());
    w.set_is_day(true);
    w.set_background(WeatherBg::Clear);
    load(app, &HANOI);
}

fn focus(app: &App, key: &str) {
    let w = app.global::<WeatherData>();
    w.set_focus(SharedString::from(key));
    w.set_focus_serial(w.get_focus_serial() + 1);
}

fn open(app: &App) {
    app.set_show_sample(false);
    app.global::<crate::Nav>().set_screen(crate::Screen::Weather);
}

pub fn background(app: &App, bg: WeatherBg, day: bool) {
    let w = app.global::<WeatherData>();
    w.set_background(bg);
    w.set_is_day(day);
    w.set_stale(false);
    w.set_preview(false);
    load(app, &HANOI);
    open(app);
}

pub fn tomorrow_rain(app: &App) {
    background(app, WeatherBg::Rain, true);
    show_day(app, &HANOI, 1);
    focus(app, "d1");
}

pub fn afternoon(app: &App) {
    open(app);
    show_day(app, &HANOI, 0);
    focus(app, "d0-h15");
}

pub fn air(app: &App) {
    open(app);
    focus(app, "aqi");
}

pub fn danang(app: &App) {
    let w = app.global::<WeatherData>();
    w.set_preview(true);
    load(app, &DANANG);
    open(app);
}

pub fn offline(app: &App) {
    let w = app.global::<WeatherData>();
    w.set_stale(true);
    w.set_updated("09:10".into());
    open(app);
}

#[cfg(test)]
#[path = "weather_tests.rs"]
mod tests;
