use crate::{Alert, UiState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertKind {
    Timer,
    Reminder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlertView {
    pub kind: AlertKind,
    pub title: String,
    pub time: String,
    pub body: String,
    pub serial: u64,
}

pub(super) fn from_state(state: &UiState) -> Option<AlertView> {
    let (kind, title, time, body) = match state.alert.as_ref()? {
        Alert::Timer(timer) => (
            AlertKind::Timer,
            "Hẹn giờ đã xong".to_string(),
            super::timers::time_at(&state.now, timer.end_seconds),
            timer.label.clone(),
        ),
        Alert::Reminder(reminder) => (
            AlertKind::Reminder,
            "Nhắc nhở".to_string(),
            state.now.strftime("%H:%M").to_string(),
            reminder.label.clone(),
        ),
    };
    Some(AlertView {
        kind,
        title,
        time,
        body,
        serial: state.events.alert,
    })
}
