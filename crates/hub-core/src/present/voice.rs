use crate::{UiState, VoiceState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceView {
    pub state: VoiceState,
    pub mic_muted: bool,
    pub wake_serial: u64,
    pub tool_serial: u64,
    pub error_serial: u64,
}

pub(super) fn from_state(state: &UiState) -> VoiceView {
    VoiceView {
        state: state.voice.state,
        mic_muted: false,
        wake_serial: state.events.wake,
        tool_serial: state.events.tool,
        error_serial: state.events.error,
    }
}
