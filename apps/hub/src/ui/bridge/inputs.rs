use crate::{App, CalendarData, Nav, SettingsData, Voice, WeatherData};
use hub_core::{Focus, Screen as CoreScreen, UserInput};
use slint::ComponentHandle;
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::sync::mpsc;

#[cfg(test)]
#[path = "inputs_tests.rs"]
mod tests;

struct Inputs {
    tx: mpsc::Sender<UserInput>,
    last_activity: Option<Instant>,
}

impl Inputs {
    fn new(tx: mpsc::Sender<UserInput>) -> Self {
        Self {
            tx,
            last_activity: None,
        }
    }

    fn send(&self, input: UserInput) {
        if let Err(error) = self.tx.try_send(input) {
            let cause = match error {
                mpsc::error::TrySendError::Full(_) => "hàng đợi đầy",
                mpsc::error::TrySendError::Closed(_) => "channel đã đóng",
            };
            eprintln!("hub-core không nhận được đầu vào UI: {cause}");
        }
    }

    fn activity(&mut self) {
        self.activity_at(Instant::now());
    }

    fn activity_at(&mut self, now: Instant) {
        let should_send = self
            .last_activity
            .is_none_or(|last| now.saturating_duration_since(last) >= Duration::from_secs(1));
        if should_send {
            self.last_activity = Some(now);
            self.send(UserInput::Activity);
        }
    }
}

fn to_core_screen(screen: crate::Screen) -> Option<CoreScreen> {
    use crate::Screen as UiScreen;
    match screen {
        UiScreen::Center => Some(CoreScreen::Center),
        UiScreen::Menu => Some(CoreScreen::Menu),
        UiScreen::Ambient => Some(CoreScreen::Ambient),
        UiScreen::Weather => Some(CoreScreen::Weather),
        UiScreen::Calendar => Some(CoreScreen::Calendar),
        UiScreen::Home => Some(CoreScreen::Home),
        UiScreen::Timers => Some(CoreScreen::Timers),
        UiScreen::Countdowns => Some(CoreScreen::Countdowns),
        UiScreen::Reminders => Some(CoreScreen::Reminders),
        UiScreen::Photos => Some(CoreScreen::Photos),
        UiScreen::Settings => Some(CoreScreen::Settings),
        UiScreen::FirstBoot => None,
    }
}

pub(super) fn to_ui_screen(screen: CoreScreen) -> crate::Screen {
    use crate::Screen as UiScreen;
    match screen {
        CoreScreen::Center => UiScreen::Center,
        CoreScreen::Menu => UiScreen::Menu,
        CoreScreen::Ambient => UiScreen::Ambient,
        CoreScreen::Weather => UiScreen::Weather,
        CoreScreen::Calendar => UiScreen::Calendar,
        CoreScreen::Home => UiScreen::Home,
        CoreScreen::Timers => UiScreen::Timers,
        CoreScreen::Countdowns => UiScreen::Countdowns,
        CoreScreen::Reminders => UiScreen::Reminders,
        CoreScreen::Photos => UiScreen::Photos,
        CoreScreen::Settings => UiScreen::Settings,
    }
}

/// Nối callback hợp đồng vào hub-core. Gọi trên luồng UI trước `app.run()`.
pub fn connect_inputs(app: &App, input_tx: mpsc::Sender<UserInput>) {
    let inputs = Rc::new(RefCell::new(Inputs::new(input_tx)));
    let nav = app.global::<Nav>();
    let activity = inputs.clone();
    nav.on_activity(move || activity.borrow_mut().activity());

    let navigate = inputs.clone();
    nav.on_navigate(move |screen| {
        if let Some(screen) = to_core_screen(screen) {
            let mut inputs = navigate.borrow_mut();
            inputs.activity();
            inputs.send(UserInput::Open(screen));
        } else {
            eprintln!("hub-core không có màn first-boot; bỏ qua điều hướng");
        }
    });

    let weather = app.global::<WeatherData>();
    let select_weather = inputs.clone();
    weather.on_select_day(move |key| {
        let mut inputs = select_weather.borrow_mut();
        inputs.activity();
        inputs.send(UserInput::Focus(Some(Focus::Weather(key.to_string()))));
    });

    let calendar = app.global::<CalendarData>();
    let weak = app.as_weak();
    let select_calendar = inputs.clone();
    calendar.on_select_day(move |day| {
        if let Some(app) = weak.upgrade() {
            super::calendar::select_day(&app);
            let mut inputs = select_calendar.borrow_mut();
            inputs.activity();
            inputs.send(UserInput::Focus(Some(Focus::Calendar(day.to_string()))));
        }
    });
    let weak = app.as_weak();
    let toggle_month = inputs.clone();
    calendar.on_toggle_month(move || {
        if let Some(app) = weak.upgrade() {
            super::calendar::toggle_month(&app);
            toggle_month.borrow_mut().activity();
        }
    });
    let weak = app.as_weak();
    let open_detail = inputs.clone();
    calendar.on_open_detail(move |key| {
        if let Some(app) = weak.upgrade() {
            super::calendar::open_detail(&app, key.to_string());
            open_detail.borrow_mut().activity();
        }
    });
    let weak = app.as_weak();
    let close_detail = inputs.clone();
    calendar.on_close_detail(move || {
        if let Some(app) = weak.upgrade() {
            super::calendar::close_detail(&app);
            close_detail.borrow_mut().activity();
        }
    });

    let orb = inputs.clone();
    app.global::<Voice>().on_orb_tapped(move || {
        let mut inputs = orb.borrow_mut();
        inputs.activity();
        inputs.send(UserInput::OrbTapped);
    });

    let settings = app.global::<SettingsData>();
    let save = inputs.clone();
    settings.on_save_voice_url(move |url| {
        save.borrow().send(UserInput::SaveVoiceUrl(url.to_string()));
    });
    let connect = inputs;
    settings.on_connect_voice(move || {
        connect.borrow().send(UserInput::VoiceConnect);
    });
}
