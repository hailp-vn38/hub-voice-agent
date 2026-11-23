use crate::UiState;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbientView {
    pub time: String,
    pub temp: String,
    pub icon: String,
    pub night: bool,
}

pub(super) fn from_state(state: &UiState) -> AmbientView {
    let forecast = state.weather.forecast.as_ref();
    let temp = forecast
        .and_then(|value| value["current"]["temperature_2m"].as_f64())
        .map(|temperature| format!("{temperature:.0}°"))
        .unwrap_or_default();

    AmbientView {
        time: state.now.strftime("%H:%M").to_string(),
        temp,
        icon: if state.weather.icon.is_empty() {
            "sun".into()
        } else {
            state.weather.icon.clone()
        },
        night: is_night(&state.now, forecast),
    }
}

fn is_night(now: &jiff::Zoned, forecast: Option<&Value>) -> bool {
    let Some(forecast) = forecast else {
        return false;
    };
    let date = now.strftime("%F").to_string();
    let date_time = now.strftime("%FT%R").to_string();
    let Some(days) = forecast["daily"]["time"].as_array() else {
        return false;
    };
    let Some(index) = days
        .iter()
        .position(|day| day.as_str() == Some(date.as_str()))
    else {
        return false;
    };
    let Some(sunrise) = forecast["daily"]["sunrise"][index].as_str() else {
        return false;
    };
    let Some(sunset) = forecast["daily"]["sunset"][index].as_str() else {
        return false;
    };
    date_time.as_str() < sunrise || date_time.as_str() >= sunset
}
