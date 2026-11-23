use crate::{UserInput, runtime::effects::Effect};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Center,
    Menu,
    Ambient,
    Weather,
    Calendar,
    Home,
    Timers,
    Countdowns,
    Reminders,
    Photos,
    Settings,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Focus {
    Timer(i64),
    Calendar(String),
    Photo(String),
    Countdown(i64),
    Weather(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ViewState {
    pub screen: Screen,
    pub focus: Option<Focus>,
}

#[derive(Default)]
pub(crate) struct View {
    state: ViewState,
    last_activity: Option<i64>,
    back_to_menu: bool,
}

impl View {
    pub fn focus(&self) -> Option<&Focus> {
        self.state.focus.as_ref()
    }
    pub fn screen(&self) -> Screen {
        self.state.screen
    }
    pub fn new(now: i64) -> Self {
        Self {
            last_activity: Some(now),
            ..Self::default()
        }
    }

    pub fn open(&mut self, screen: Screen, focus: Option<Focus>, now: i64) -> Vec<Effect> {
        if screen != Screen::Menu && !matches!(screen, Screen::Center | Screen::Ambient) {
            if self.state.screen == Screen::Menu {
                self.back_to_menu = true;
            } else if matches!(self.state.screen, Screen::Center | Screen::Ambient) {
                self.back_to_menu = false;
            }
        }
        let place_focus = focus.is_some();
        self.state = ViewState { screen, focus };
        self.last_activity = Some(now);
        self.snapshot(place_focus)
    }

    /// Opens `screen` for a voice tool, keeping the focus when it is already open.
    pub fn show(&mut self, screen: Screen, now: i64) -> Vec<Effect> {
        let focus = if self.state.screen == screen {
            self.state.focus.clone()
        } else {
            None
        };
        self.open(screen, focus, now)
    }

    pub fn on_user(&mut self, input: UserInput, now: i64) -> Vec<Effect> {
        match input {
            UserInput::Open(screen) => self.open(screen, None, now),
            UserInput::Back => {
                let target = if self.back_to_menu
                    && !matches!(
                        self.state.screen,
                        Screen::Menu | Screen::Center | Screen::Ambient
                    ) {
                    Screen::Menu
                } else {
                    Screen::Center
                };
                self.open(target, None, now)
            }
            UserInput::Activity if self.state.screen == Screen::Ambient => {
                self.open(Screen::Center, None, now)
            }
            UserInput::Focus(focus) => {
                self.state.focus = focus;
                self.last_activity = Some(now);
                self.snapshot(false)
            }
            UserInput::PinPhoto { .. }
            | UserInput::ToggleHome(_)
            | UserInput::RemovePhoto { .. }
            | UserInput::RemoveCountdown(_)
            | UserInput::SaveVoiceUrl(_)
            | UserInput::VoiceConnect
            | UserInput::OrbTapped
            | UserInput::Setup(_, _)
            | UserInput::Timer(_)
            | UserInput::DismissAlert => Vec::new(),
            UserInput::Activity => {
                self.last_activity = Some(now);
                Vec::new()
            }
        }
    }

    pub fn tick(&mut self, now: i64) -> Vec<Effect> {
        if let Some(last) = self.last_activity {
            let idle = now.saturating_sub(last);
            if idle >= 300 && self.state.screen != Screen::Ambient {
                self.state = ViewState {
                    screen: Screen::Ambient,
                    focus: None,
                };
                return self.snapshot(false);
            }
            if idle >= 60 && !matches!(self.state.screen, Screen::Center | Screen::Ambient) {
                self.state = ViewState::default();
                return self.snapshot(false);
            }
        }
        Vec::new()
    }

    fn snapshot(&self, focus_event: bool) -> Vec<Effect> {
        let view = self.state.clone();
        vec![Effect::Ui(Box::new(move |ui| {
            let view_changed = ui.view != view;
            if view_changed {
                ui.view = view;
            }
            if focus_event {
                ui.events.focus = ui.events.focus.saturating_add(1);
            }
            view_changed || focus_event
        }))]
    }
}

/// The screen a Device MCP tool drives (ADR 0008).
pub(crate) fn screen_for_hub_tool(name: &str) -> Option<Screen> {
    const PREFIXES: [(&str, Screen); 8] = [
        ("hub_calendar_", Screen::Calendar),
        ("hub_timer_", Screen::Timers),
        ("hub_countdown_", Screen::Countdowns),
        ("hub_reminder_", Screen::Reminders),
        ("hub_weather_", Screen::Weather),
        ("hub_home_", Screen::Home),
        ("hub_image_", Screen::Photos),
        ("hub_show_image", Screen::Photos),
    ];
    PREFIXES
        .iter()
        .find(|(prefix, _)| name.starts_with(prefix))
        .map(|(_, screen)| *screen)
}

/// The screen a server-side tool opens. The hub only learns the tool's name, and
/// names come from the server's configuration, so they are matched by keyword;
/// an unknown name leaves the view as it is.
pub(crate) fn screen_for_server_tool(name: &str) -> Option<Screen> {
    const KEYWORDS: [(&str, Screen); 10] = [
        ("calendar", Screen::Calendar),
        ("event", Screen::Calendar),
        ("weather", Screen::Weather),
        ("forecast", Screen::Weather),
        ("hass", Screen::Home),
        ("home", Screen::Home),
        ("light", Screen::Home),
        ("switch", Screen::Home),
        ("image", Screen::Photos),
        ("photo", Screen::Photos),
    ];
    let name = name.to_ascii_lowercase();
    KEYWORDS
        .iter()
        .find(|(keyword, _)| name.contains(keyword))
        .map(|(_, screen)| *screen)
}

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
