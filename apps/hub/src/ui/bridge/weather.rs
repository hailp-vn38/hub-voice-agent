use crate::{App, WeatherBg, WeatherData, WeatherDay, WeatherHour, WeatherNow};
use hub_core::present::{WeatherBackground, WeatherDataView, WeatherDayView, WeatherHourView};
use slint::{Color, ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

pub(super) fn apply(app: &App, view: &WeatherDataView) {
    let weather = app.global::<WeatherData>();
    weather.set_location(view.location.as_str().into());
    weather.set_preview(view.preview);
    weather.set_updated(view.updated.as_str().into());
    weather.set_stale(view.stale);
    weather.set_background(match view.background {
        WeatherBackground::Clear => WeatherBg::Clear,
        WeatherBackground::Cloudy => WeatherBg::Cloudy,
        WeatherBackground::Rain => WeatherBg::Rain,
        WeatherBackground::Storm => WeatherBg::Storm,
        WeatherBackground::Fog => WeatherBg::Fog,
    });
    weather.set_is_day(view.is_day);
    weather.set_now(WeatherNow {
        icon: view.now.icon.as_str().into(),
        temp: view.now.temp.as_str().into(),
        summary: view.now.summary.as_str().into(),
        feels: view.now.feels.as_str().into(),
        humidity: view.now.humidity.as_str().into(),
        wind: view.now.wind.as_str().into(),
        uv: view.now.uv.as_str().into(),
        aqi: view.now.aqi.as_str().into(),
        aqi_level: view.now.aqi_level.as_str().into(),
        aqi_color: color_from_hex(&view.now.aqi_color),
        sunrise: view.now.sunrise.as_str().into(),
        sunset: view.now.sunset.as_str().into(),
    });
    weather.set_hours(model(view.hours.iter().map(hour).collect()));
    weather.set_selected_day(view.selected_day.as_str().into());
    weather.set_days(model(view.days.iter().map(day).collect()));
    weather.set_week_min(view.week_min);
    weather.set_week_max(view.week_max);
    weather.set_focus(view.focus.as_str().into());
    weather.set_focus_serial(view.focus_serial as i32);
}

fn hour(hour: &WeatherHourView) -> WeatherHour {
    WeatherHour {
        key: hour.key.as_str().into(),
        label: hour.label.as_str().into(),
        temp: hour.temp,
        rain: hour.rain,
        peak: hour.peak,
        wind: hour.wind.as_str().into(),
        icon: hour.icon.as_str().into(),
    }
}

fn day(day: &WeatherDayView) -> WeatherDay {
    WeatherDay {
        key: day.key.as_str().into(),
        label: day.label.as_str().into(),
        icon: day.icon.as_str().into(),
        low: day.low,
        high: day.high,
        low_text: day.low_text.as_str().into(),
        high_text: day.high_text.as_str().into(),
        is_tomorrow: day.is_tomorrow,
    }
}

fn color_from_hex(hex: &str) -> Color {
    let Some(rgb) = hex.strip_prefix('#').filter(|rgb| rgb.len() == 6) else {
        return Color::default();
    };
    let Ok(rgb) = u32::from_str_radix(rgb, 16) else {
        return Color::default();
    };
    if rgb > 0x00ff_ffff {
        return Color::default();
    }
    Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

fn model<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

#[cfg(test)]
#[path = "weather_tests.rs"]
mod tests;
