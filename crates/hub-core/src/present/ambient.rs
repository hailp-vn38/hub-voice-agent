use super::weather::{icon, is_day_at};
use crate::UiState;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbientView {
    pub time: String,
    pub temp: String,
    pub icon: String,
    pub night: bool,
}

pub(super) fn from_state(state: &UiState) -> AmbientView {
    let forecast = state.weather.forecast.as_ref();
    let temperature = forecast.and_then(|forecast| forecast.current.temperature);
    let today = state.now.strftime("%F").to_string();
    let today_forecast =
        forecast.and_then(|forecast| forecast.days.iter().find(|day| day.date == today));

    AmbientView {
        time: state.now.strftime("%H:%M").to_string(),
        temp: temperature
            .map(|temperature| format!("{temperature:.0}°"))
            .unwrap_or_default(),
        icon: forecast
            .map(|forecast| icon(forecast.current.weather_code).to_owned())
            .unwrap_or_else(|| "sun".into()),
        night: today_forecast.is_some() && !is_day_at(&state.now, today_forecast),
    }
}
