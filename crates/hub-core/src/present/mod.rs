mod ambient;
mod countdowns;
mod nav;
mod reminders;
mod status;

use crate::UiState;
pub use ambient::AmbientView;
pub use countdowns::{CountdownItemView, CountdownUnit, CountdownsView};
pub use reminders::{
    ReminderGroupView, ReminderItemView, ReminderNextView, ReminderPastView, RemindersView,
};
pub use status::StatusView;

/// Rust representation of the UI contract fields prepared by hub-core.
#[derive(Clone, Debug, PartialEq)]
pub struct HubView {
    pub nav: crate::Screen,
    pub status: StatusView,
    pub ambient: AmbientView,
    pub countdowns: CountdownsView,
    pub reminders: RemindersView,
}

/// Prepares display-ready contract data from the actor snapshot.
pub fn present(state: &UiState) -> HubView {
    HubView {
        nav: nav::screen(state),
        status: status::from_state(state),
        ambient: ambient::from_state(state),
        countdowns: countdowns::from_state(state),
        reminders: reminders::from_state(state),
    }
}
