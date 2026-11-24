//! Compact, voice-friendly weather payload (raw Open-Meteo JSON is too large for the model).
use super::model::{WeatherForecast, condition};
use serde_json::{Value, json};

fn clock(time: &str) -> &str {
    time.get(11..16).unwrap_or(time)
}

/// Summarises a raw forecast: current conditions, today's and tomorrow's
/// 3-hourly outlook, 7 daily ranges and air quality.
pub(super) fn compact(data: &Value) -> Value {
    let forecast = WeatherForecast::from_open_meteo(data);
    let current = &forecast.current;
    let today = forecast.days.first().map(|day| day.date.as_str());
    let tomorrow = forecast.days.get(1).map(|day| day.date.as_str());
    let hours: Vec<Value> = forecast
        .hours
        .iter()
        .filter(|hour| {
            let date = hour.time.get(..10);
            (date == today || date == tomorrow)
                && hour.time.get(14..16) == Some("00")
                && hour
                    .time
                    .get(11..13)
                    .and_then(|h| h.parse::<u8>().ok())
                    .is_some_and(|h| h % 3 == 0)
        })
        .map(|hour| {
            json!({
                "time": hour.time.replace('T', " "),
                "temperature_c": hour.temperature,
                "rain_chance_percent": hour.precipitation_probability,
                "condition": condition(hour.weather_code),
            })
        })
        .collect();
    let days: Vec<Value> = forecast
        .days
        .iter()
        .map(|day| {
            json!({
                "date": day.date,
                "min_c": day.temperature_min,
                "max_c": day.temperature_max,
                "condition": condition(day.weather_code),
                "sunrise": day.sunrise.as_deref().map(clock),
                "sunset": day.sunset.as_deref().map(clock),
                "uv_max": day.uv_index_max,
            })
        })
        .collect();
    let mut out = json!({
        "current": {
            "time": current.time.replace('T', " "),
            "temperature_c": current.temperature,
            "feels_like_c": current.apparent_temperature,
            "humidity_percent": current.relative_humidity,
            "wind_kmh": current.wind_speed,
            "uv_index": current.uv_index,
            "condition": condition(current.weather_code),
        },
        "air_quality": forecast.air_quality.map(|air| json!({"us_aqi": air.us_aqi, "pm2_5": air.pm2_5})),
        "hourly_today_and_tomorrow": hours,
        "daily": days,
    });
    if let Some(location) = data["location"].as_str() {
        out["location"] = location.into();
    }
    out
}
