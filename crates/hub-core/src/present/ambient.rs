use super::weather::icon;
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

    AmbientView {
        time: state.now.strftime("%H:%M").to_string(),
        temp: temperature
            .map(|temperature| format!("{temperature:.0}°"))
            .unwrap_or_default(),
        icon: forecast
            .map(|forecast| icon(forecast.current.weather_code).to_owned())
            .unwrap_or_else(|| "sun".into()),
        night: state.settings.night_now(&state.now),
    }
}
