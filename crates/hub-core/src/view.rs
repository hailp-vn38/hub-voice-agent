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
    pub fn open(&mut self, screen: Screen, focus: Option<Focus>, now: i64) -> Vec<Effect> {
        if screen != Screen::Menu && !matches!(screen, Screen::Center | Screen::Ambient) {
            if self.state.screen == Screen::Menu {
                self.back_to_menu = true;
            } else if matches!(self.state.screen, Screen::Center | Screen::Ambient) {
                self.back_to_menu = false;
            }
        }
        self.state = ViewState { screen, focus };
        self.last_activity = Some(now);
        self.snapshot()
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
            UserInput::SaveVoiceUrl(_) | UserInput::VoiceConnect | UserInput::Setup(_, _) => {
                Vec::new()
            }
            UserInput::Activity => {
                self.last_activity = Some(now);
                Vec::new()
            }
        }
    }

    pub fn tick(&mut self, now: i64) -> Vec<Effect> {
        if !matches!(self.state.screen, Screen::Center | Screen::Ambient)
            && self
                .last_activity
                .is_some_and(|last| now.saturating_sub(last) >= 60)
        {
            return self.open(Screen::Center, None, now);
        }
        Vec::new()
    }

    fn snapshot(&self) -> Vec<Effect> {
        let view = self.state.clone();
        vec![Effect::Ui(Box::new(move |ui| {
            if ui.view == view {
                return false;
            }
            ui.view = view;
            true
        }))]
    }
}
