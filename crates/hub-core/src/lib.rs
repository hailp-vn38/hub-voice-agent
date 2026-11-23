//! Hub core: the actor that owns state and drives the UI snapshot. Depends on `voice-client`.

mod alerts;
mod audio;
mod config;
mod conversation;
mod features;
mod lunar;
mod selector;
mod sources;

mod ports;
pub mod present;
mod runtime;
pub mod setup_web;
mod store;
mod ui_state;
mod view;
mod voice_settings;

use std::sync::Arc;

pub use alerts::Alert;
pub use audio::{AudioCommand, AudioEvent, Sensitivity, Sound};
pub use config::{
    CalendarSettings, Config, ConfigStore, HomeSettings, VoiceSettings, WeatherSettings,
};
pub use ports::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarQuery, CalendarSource, Clock,
    FetchedImage, HomeDomain, HomeEntity, HomeEvent, HomeSource, ImageFetcher, NewCalendarEvent,
    System, SystemClock, WeatherLocation, WeatherSource,
};
pub use present::{
    AlertKind, AlertView, CalendarBlockView, CalendarDayView, CalendarDetailView, CalendarKind,
    CalendarMonthDayView, CalendarUpcomingView, CalendarView, CenterCard, CenterCountdown,
    CenterEvent, CenterReminder, CenterTimer, CenterView, CenterWeather, CountdownItemView,
    CountdownUnit, CountdownsView, HomeKind, HomeTileView, HomeView, HubView, MenuView,
    PendingState, PhotoState, PhotoView, PhotosView, ReminderGroupView, ReminderItemView,
    ReminderNextView, ReminderPastView, RemindersView, TimerCardView, TimerPhase, TimersView,
    WeatherBackground, WeatherDataView, WeatherDayView, WeatherHourView, WeatherNowView, present,
};
pub use runtime::run;
use tokio::sync::{mpsc, watch};
pub use ui_state::{
    PhotoFetch, PhotoFetchState, ResponseSpeaker, ToolTrailState, ToolTrailStep, UiEvents, UiState,
    VoiceSettingsState, VoiceState, VoiceUiState,
};
pub use view::{Focus, Screen, ViewState};
use voice_client::{VoiceCommand, VoiceConfig, VoiceEvent};
pub use voice_settings::DEFAULT_OTA_URL;

pub struct Ports {
    pub clock: Arc<dyn Clock>,
    pub calendar: Arc<dyn CalendarSource>,
    pub weather: Arc<dyn WeatherSource>,
    pub home: Arc<dyn HomeSource>,
    pub images: Arc<dyn ImageFetcher>,
    pub system: Arc<dyn System>,
}

pub enum UserInput {
    Focus(Option<Focus>),
    Timer(TimerInput),
    DismissAlert,
    PinPhoto {
        id: String,
        pinned: bool,
    },
    ToggleHome(String),
    RemovePhoto {
        id: String,
    },
    RemoveCountdown(i64),
    Open(Screen),
    Back,
    Activity,
    OrbTapped,
    SaveVoiceUrl(String),
    VoiceConnect,
    Setup(
        setup_web::Update,
        tokio::sync::oneshot::Sender<Result<(), String>>,
    ),
}

pub enum TimerInput {
    QuickStart(u32),
    TogglePause(i64),
    AddMinute(i64),
    Cancel(i64),
}

pub struct Io {
    pub voice_config_tx: watch::Sender<Option<VoiceConfig>>,
    pub voice_tx: mpsc::Sender<VoiceCommand>,
    pub voice_rx: mpsc::Receiver<VoiceEvent>,
    pub audio_tx: mpsc::Sender<AudioCommand>,
    pub audio_rx: mpsc::Receiver<AudioEvent>,
    pub input_rx: mpsc::Receiver<UserInput>,
    pub ui_tx: watch::Sender<UiState>,
}

pub use features::countdowns::{Countdown, CountdownView, Repeat};
pub use features::photos::Photo;
pub use features::reminders::{Reminder, ReminderDue, ReminderHistory, ReminderRepeat};
pub use features::timers::Timer;
pub use features::weather::{
    AirQuality, WeatherCurrent, WeatherDay, WeatherForecast, WeatherHour, WeatherView,
};
pub use lunar::{LunarDate, to_lunar, to_solar};
pub use sources::{GoogleCalendar, HomeAssistant, HttpImageFetcher, OpenMeteo};
pub use store::Store;

pub fn tool_definitions() -> Vec<voice_client::ToolDef> {
    let mut tools = features::timers::tools::definitions();
    tools.extend(features::countdowns::tools::definitions());
    tools.extend(features::reminders::tools::definitions());
    tools.extend(features::calendar::tools::definitions());
    tools.extend(features::weather::tools::definitions());
    tools.extend(features::home::tools::definitions());
    tools.extend(features::photos::tools::definitions());
    tools
}
