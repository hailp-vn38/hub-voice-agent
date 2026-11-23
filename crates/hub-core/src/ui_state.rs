/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct UiState {
    /// Current local time supplied by the actor's clock port.
    pub now: jiff::Zoned,
    /// Monotonic serials for one-shot UI effects.
    pub events: UiEvents,
    pub calendar: Vec<crate::CalendarEvent>,
    pub calendar_online: Option<bool>,
    pub calendar_last_sync: Option<i64>,
    pub weather: crate::WeatherView,
    pub home: Vec<crate::HomeEntity>,
    pub photos: Vec<crate::Photo>,
    pub alert: Option<crate::Alert>,
    pub reminders: Vec<crate::Reminder>,
    pub reminder_history: Vec<crate::ReminderHistory>,
    pub countdowns: Vec<crate::CountdownView>,
    pub view: crate::ViewState,
    pub voice_settings: VoiceSettingsState,
    pub timers: Vec<crate::Timer>,
    pub voice: VoiceUiState,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            now: "1970-01-01T00:00:00+00:00[UTC]"
                .parse()
                .expect("the default timestamp is valid"),
            events: UiEvents::default(),
            calendar: Vec::new(),
            calendar_online: None,
            calendar_last_sync: None,
            weather: crate::WeatherView::default(),
            home: Vec::new(),
            photos: Vec::new(),
            alert: None,
            reminders: Vec::new(),
            reminder_history: Vec::new(),
            countdowns: Vec::new(),
            view: crate::ViewState::default(),
            voice_settings: VoiceSettingsState::default(),
            timers: Vec::new(),
            voice: VoiceUiState::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiEvents {
    pub focus: u64,
    pub wake: u64,
    pub tool: u64,
    pub error: u64,
    pub alert: u64,
    pub confetti: u64,
    pub spawn: u64,
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
