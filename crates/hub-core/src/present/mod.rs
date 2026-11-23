mod alerts;
mod ambient;
mod nav;
mod status;
mod timers;
mod weather;

use crate::UiState;
pub use alerts::{AlertKind, AlertView};
pub use ambient::AmbientView;
pub use status::StatusView;
pub use timers::{TimerCardView, TimerPhase, TimersView};
pub use weather::{
    WeatherBackground, WeatherDataView, WeatherDayView, WeatherHourView, WeatherNowView,
};

/// Rust representation of the UI contract fields prepared by hub-core.
#[derive(Clone, Debug, PartialEq)]
pub struct HubView {
    pub nav: crate::Screen,
    pub status: StatusView,
    pub ambient: AmbientView,
    pub timers: TimersView,
    pub alert: Option<AlertView>,
    pub weather: WeatherDataView,
}

/// Prepares display-ready contract data from the actor snapshot.
pub fn present(state: &UiState) -> HubView {
    HubView {
        nav: nav::screen(state),
        status: status::from_state(state),
        ambient: ambient::from_state(state),
        timers: timers::from_state(state),
        alert: alerts::from_state(state),
        weather: weather::from_state(state),
    }
}
