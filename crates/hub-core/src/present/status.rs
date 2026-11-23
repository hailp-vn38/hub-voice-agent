use crate::{UiState, VoiceState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusView {
    pub time: String,
    pub offline: bool,
}

pub(super) fn from_state(state: &UiState) -> StatusView {
    StatusView {
        time: state.now.strftime("%H:%M").to_string(),
        offline: state.voice.state == VoiceState::Offline,
    }
}
