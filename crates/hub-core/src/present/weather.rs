use crate::{Focus, UiState, WeatherDay, WeatherForecast, WeatherHour};
use jiff::Zoned;

mod format;
use format::{
    aqi_view, background, clock, day_label, number, percent, speed, summary, temperature,
    updated_time,
};
pub(super) use format::{icon, is_day_at};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeatherBackground {
    #[default]
    Clear,
    Cloudy,
    Rain,
    Storm,
    Fog,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherDataView {
    pub location: String,
    pub preview: bool,
    pub updated: String,
    pub stale: bool,
    pub background: WeatherBackground,
    pub is_day: bool,
    pub now: WeatherNowView,
    pub hours: Vec<WeatherHourView>,
    pub selected_day: String,
    pub days: Vec<WeatherDayView>,
    pub week_min: f32,
    pub week_max: f32,
    pub focus: String,
    pub focus_serial: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherNowView {
    pub icon: String,
    pub temp: String,
    pub summary: String,
    pub feels: String,
    pub humidity: String,
    pub wind: String,
    pub uv: String,
    pub aqi: String,
    pub aqi_level: String,
    pub aqi_color: String,
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherHourView {
    pub key: String,
    pub label: String,
    pub temp: f32,
    pub rain: f32,
    pub wind: String,
    pub icon: String,
    pub peak: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherDayView {
    pub key: String,
    pub label: String,
    pub icon: String,
    pub low: f32,
    pub high: f32,
    pub low_text: String,
    pub high_text: String,
    pub is_tomorrow: bool,
}

pub(super) fn from_state(state: &UiState) -> WeatherDataView {
    let forecast = state.weather.forecast.as_ref();
    let today = state.now.strftime("%F").to_string();
    let focus = match state.view.focus.as_ref() {
        Some(Focus::Weather(key)) => Some(key.clone()),
        _ => None,
    };
    let selected_day = focus
        .as_deref()
        .filter(|key| *key != "aqi")
        .and_then(|key| key.get(..10))
        .filter(|date| date.parse::<jiff::civil::Date>().is_ok())
        .unwrap_or(&today)
        .to_owned();
    let selected_hours: Vec<_> = forecast
        .into_iter()
        .flat_map(|forecast| &forecast.hours)
        .filter(|hour| hour.time.starts_with(&selected_day))
        .collect();
    let peak = selected_hours
        .iter()
        .filter_map(|hour| hour.temperature)
        .reduce(f32::max);
    let hours = selected_hours
        .iter()
        .map(|hour| hour_view(hour, peak))
        .collect();
    let days = forecast
        .map(|forecast| {
            forecast
                .days
                .iter()
                .map(|day| day_view(day, &today, &state.now))
                .collect()
        })
        .unwrap_or_default();
    let week_min = forecast
        .into_iter()
        .flat_map(|forecast| &forecast.days)
        .filter_map(|day| day.temperature_min)
        .reduce(f32::min)
        .unwrap_or_default();
    let week_max = forecast
        .into_iter()
        .flat_map(|forecast| &forecast.days)
        .filter_map(|day| day.temperature_max)
        .reduce(f32::max)
        .unwrap_or_default();
    let today_forecast =
        forecast.and_then(|forecast| forecast.days.iter().find(|day| day.date == today));
    let code = forecast.and_then(|forecast| forecast.current.weather_code);

    WeatherDataView {
        location: state.weather.location.clone().unwrap_or_default(),
        preview: state.weather.preview,
        updated: updated_time(&state.now, state.weather.updated),
        stale: state.weather.stale,
        background: background(code),
        is_day: is_day_at(&state.now, today_forecast),
        now: now_view(forecast, today_forecast),
        hours,
        selected_day,
        days,
        week_min,
        week_max,
        focus: focus.unwrap_or_default(),
        focus_serial: state.events.focus,
    }
}

fn now_view(forecast: Option<&WeatherForecast>, today: Option<&WeatherDay>) -> WeatherNowView {
    let Some(forecast) = forecast else {
        return WeatherNowView::default();
    };
    let current = &forecast.current;
    let code = current.weather_code;
    let uv = current.uv_index.or_else(|| {
        forecast
            .hours
            .iter()
            .find(|hour| {
                current
                    .time
                    .get(..13)
                    .is_some_and(|prefix| hour.time.starts_with(prefix))
            })
            .and_then(|hour| hour.uv_index)
    });
    let (aqi, aqi_level, aqi_color) = forecast
        .air_quality
        .as_ref()
        .and_then(|air| air.us_aqi)
        .map(aqi_view)
        .unwrap_or_default();
    WeatherNowView {
        icon: icon(code).into(),
        temp: temperature(current.temperature),
        summary: summary(code).into(),
        feels: temperature(current.apparent_temperature),
        humidity: percent(current.relative_humidity),
        wind: speed(current.wind_speed),
        uv: number(uv),
        aqi,
        aqi_level,
        aqi_color,
        sunrise: today
            .and_then(|day| clock(&day.sunrise))
            .unwrap_or_default(),
        sunset: today.and_then(|day| clock(&day.sunset)).unwrap_or_default(),
    }
}

fn hour_view(hour: &WeatherHour, peak: Option<f32>) -> WeatherHourView {
    WeatherHourView {
        key: hour.time.clone(),
        label: hour.time.get(11..16).unwrap_or_default().to_owned(),
        temp: hour.temperature.unwrap_or_default(),
        rain: hour.precipitation_probability.unwrap_or_default(),
        wind: speed(hour.wind_speed),
        icon: icon(hour.weather_code).into(),
        peak: hour.temperature.is_some() && hour.temperature == peak,
    }
}

fn day_view(day: &WeatherDay, today: &str, now: &Zoned) -> WeatherDayView {
    let tomorrow = format::tomorrow(now);
    let low = day.temperature_min.unwrap_or_default();
    let high = day.temperature_max.unwrap_or_default();
    WeatherDayView {
        key: day.date.clone(),
        label: day_label(&day.date, today, tomorrow.as_deref()),
        icon: icon(day.weather_code).into(),
        low,
        high,
        low_text: temperature(Some(low)),
        high_text: temperature(Some(high)),
        is_tomorrow: tomorrow.as_deref() == Some(day.date.as_str()),
    }
}
