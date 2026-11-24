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

impl Screen {
    /// Every screen; the UI bridge tests round-trip each one through Slint.
    pub const ALL: [Screen; 11] = [
        Screen::Center,
        Screen::Menu,
        Screen::Ambient,
        Screen::Weather,
        Screen::Calendar,
        Screen::Home,
        Screen::Timers,
        Screen::Countdowns,
        Screen::Reminders,
        Screen::Photos,
        Screen::Settings,
    ];
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
            | UserInput::Setting(_)
            | UserInput::KeepResolution
            | UserInput::PreviewSound(_)
            | UserInput::Clear(_)
            | UserInput::Power(_)
            | UserInput::ReminderSpeech(_)
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

    /// `ambient_after`: số giây không chạm để vào Ambient, `None` = không vào.
    pub fn tick(&mut self, now: i64, ambient_after: Option<i64>) -> Vec<Effect> {
        if let Some(last) = self.last_activity {
            let idle = now.saturating_sub(last);
            if ambient_after.is_some_and(|after| idle >= after)
                && self.state.screen != Screen::Ambient
            {
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

#[cfg(test)]
#[path = "view_tests.rs"]
mod tests;
