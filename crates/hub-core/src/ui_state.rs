/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiState {
    /// Local time as `HH:MM:SS`.
    pub clock: String,
    pub voice_settings: VoiceSettingsState,
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
