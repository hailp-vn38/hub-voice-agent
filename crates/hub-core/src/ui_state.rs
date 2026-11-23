/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiState {
    /// Local time as `HH:MM:SS`.
    pub clock: String,
    pub view: crate::ViewState,
    pub timers: Vec<crate::Timer>,
    pub voice: VoiceUiState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceState {
    #[default]
    Offline,
    Idle,
    Listening,
    Thinking,
    Speaking,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VoiceUiState {
    pub state: VoiceState,
    pub stt_text: String,
    pub response_text: String,
}
