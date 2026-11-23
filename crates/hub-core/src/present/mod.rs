mod alerts;
mod ambient;
mod countdowns;
mod nav;
mod reminders;
mod response;
mod status;
mod timers;
mod tool_trail;
mod voice;

use crate::UiState;
pub use alerts::{AlertKind, AlertView};
pub use ambient::AmbientView;
pub use countdowns::{CountdownItemView, CountdownUnit, CountdownsView};
pub use reminders::{
    ReminderGroupView, ReminderItemView, ReminderNextView, ReminderPastView, RemindersView,
};
pub use response::{ResponseView, Word};
pub use status::StatusView;
pub use timers::{TimerCardView, TimerPhase, TimersView};
pub use tool_trail::ToolTrailView;
pub use voice::VoiceView;

/// Rust representation of the UI contract fields prepared by hub-core.
#[derive(Clone, Debug, PartialEq)]
pub struct HubView {
    pub nav: crate::Screen,
    pub status: StatusView,
    pub ambient: AmbientView,
    pub countdowns: CountdownsView,
    pub reminders: RemindersView,
    pub timers: TimersView,
    pub alert: Option<AlertView>,
    pub voice: VoiceView,
    pub response: ResponseView,
    pub tool_trail: ToolTrailView,
}

/// Prepares display-ready contract data from the actor snapshot.
pub fn present(state: &UiState) -> HubView {
    HubView {
        nav: nav::screen(state),
        status: status::from_state(state),
        ambient: ambient::from_state(state),
        countdowns: countdowns::from_state(state),
        reminders: reminders::from_state(state),
        timers: timers::from_state(state),
        alert: alerts::from_state(state),
        voice: voice::from_state(state),
        response: response::from_state(state),
        tool_trail: tool_trail::from_state(state),
    }
}
