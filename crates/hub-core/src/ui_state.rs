/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
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
    pub home_connected: bool,
    pub home_configured: bool,
    pub home_pending: Vec<String>,
    pub home_last_updated: Option<i64>,
    pub photos: Vec<crate::Photo>,
    pub photo_fetches: Vec<PhotoFetch>,
    pub alert: Option<crate::Alert>,
    pub reminders: Vec<crate::Reminder>,
    pub reminder_history: Vec<crate::ReminderHistory>,
    pub countdowns: Vec<crate::CountdownView>,
    pub view: crate::ViewState,
    pub voice_settings: VoiceSettingsState,
    pub timers: Vec<crate::Timer>,
    pub voice: VoiceUiState,
    pub tool_trail: Vec<ToolTrailStep>,
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
            home_connected: false,
            home_configured: false,
            home_pending: Vec::new(),
            home_last_updated: None,
            photos: Vec::new(),
            photo_fetches: Vec::new(),
            alert: None,
            reminders: Vec::new(),
            reminder_history: Vec::new(),
            countdowns: Vec::new(),
            view: crate::ViewState::default(),
            voice_settings: VoiceSettingsState::default(),
            timers: Vec::new(),
            voice: VoiceUiState::default(),
            tool_trail: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhotoFetch {
    pub key: String,
    pub caption: String,
    pub source: String,
    pub created: i64,
    pub state: PhotoFetchState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoFetchState {
    Loading,
    Failed,
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
    pub response_speaker: ResponseSpeaker,
    pub response_line_serial: u64,
    pub response_fading: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResponseSpeaker {
    User,
    #[default]
    Agent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolTrailState {
    Running,
    Done,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolTrailStep {
    pub key: String,
    pub label: String,
    pub state: ToolTrailState,
}
