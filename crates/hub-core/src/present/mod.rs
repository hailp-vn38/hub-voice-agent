mod alerts;
mod ambient;
mod calendar;
mod center;
mod countdowns;
mod date_format;
mod home;
mod menu;
mod nav;
mod photos;
mod reminders;
mod response;
mod status;
mod timers;
mod tool_trail;
mod voice;
mod weather;

use crate::UiState;
pub use alerts::{AlertKind, AlertView};
pub use ambient::AmbientView;
pub use calendar::{
    CalendarBlockView, CalendarDayView, CalendarDetailView, CalendarKind, CalendarMonthDayView,
    CalendarUpcomingView, CalendarView, PendingState,
};
pub use center::{
    CenterCard, CenterCountdown, CenterEvent, CenterReminder, CenterTimer, CenterView,
    CenterWeather,
};
pub use countdowns::{CountdownItemView, CountdownUnit, CountdownsView};
pub use home::{HomeKind, HomeTileView, HomeView};
pub use menu::MenuView;
pub use photos::{PhotoState, PhotoView, PhotosView};
pub use reminders::{
    ReminderGroupView, ReminderItemView, ReminderNextView, ReminderPastView, RemindersView,
};
pub use response::{ResponseView, Word};
pub use status::StatusView;
pub use timers::{TimerCardView, TimerPhase, TimersView};
pub use tool_trail::ToolTrailView;
pub use voice::VoiceView;
pub use weather::{
    WeatherBackground, WeatherDataView, WeatherDayView, WeatherHourView, WeatherNowView,
};

/// Rust representation of the UI contract fields prepared by hub-core.
#[derive(Clone, Debug, PartialEq)]
pub struct HubView {
    pub nav: crate::Screen,
    pub center: CenterView,
    pub menu: MenuView,
    pub status: StatusView,
    pub ambient: AmbientView,
    pub calendar: CalendarView,
    pub countdowns: CountdownsView,
    pub home: HomeView,
    pub photos: PhotosView,
    pub reminders: RemindersView,
    pub timers: TimersView,
    pub alert: Option<AlertView>,
    pub voice: VoiceView,
    pub response: ResponseView,
    pub tool_trail: ToolTrailView,
    pub weather: WeatherDataView,
}

/// Prepares display-ready contract data from the actor snapshot.
pub fn present(state: &UiState) -> HubView {
    let calendar = calendar::from_state(state);
    let countdowns = countdowns::from_state(state);
    let home = home::from_state(state);
    let photos = photos::from_state(state);
    let reminders = reminders::from_state(state);
    let timers = timers::from_state(state);
    let weather = weather::from_state(state);
    let center = center::from_state(state, &countdowns, &home, &reminders, &timers, &weather);
    let menu = menu::from_state(state, &center, &countdowns, &home, &photos, &reminders);
    HubView {
        nav: nav::screen(state),
        status: status::from_state(state),
        ambient: ambient::from_state(state),
        center,
        menu,
        calendar,
        countdowns,
        home,
        photos,
        reminders,
        timers,
        alert: alerts::from_state(state),
        voice: voice::from_state(state),
        response: response::from_state(state),
        tool_trail: tool_trail::from_state(state),
        weather,
    }
}
