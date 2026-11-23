/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiState {
    /// Local time as `HH:MM:SS`.
    pub clock: String,
    pub calendar: Vec<crate::CalendarEvent>,
    pub countdowns: Vec<crate::CountdownView>,
    pub view: crate::ViewState,
    pub voice_settings: VoiceSettingsState,
    pub timers: Vec<crate::Timer>,
    pub voice: VoiceUiState,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VoiceSettingsState {
    pub ota_url: String,
    pub device_id: String,
    pub status_label: String,
    pub activation: Option<String>,
    pub error: Option<String>,
    pub enrolling: bool,
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
