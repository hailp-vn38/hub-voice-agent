use super::BridgeState;
use crate::{AlertData, App, CalendarData, Nav, SettingsData, TimersData, Voice, WeatherData};
use hub_core::{Focus, Screen as CoreScreen, TimerInput, UserInput};
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

pub(super) struct Inputs {
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

    pub(super) fn send(&self, input: UserInput) {
        if let Err(error) = self.tx.try_send(input) {
            let cause = match error {
                mpsc::error::TrySendError::Full(_) => "hàng đợi đầy",
                mpsc::error::TrySendError::Closed(_) => "channel đã đóng",
            };
            eprintln!("hub-core không nhận được đầu vào UI: {cause}");
        }
    }

    pub(super) fn activity(&mut self) {
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

pub(super) fn to_core_screen(screen: crate::Screen) -> Option<CoreScreen> {
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

fn parse_timer_id(key: &str) -> Option<i64> {
    match key.parse() {
        Ok(id) => Some(id),
        Err(error) => {
            eprintln!("id hẹn giờ không hợp lệ {key:?}: {error}");
            None
        }
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
pub fn connect_inputs(app: &App, input_tx: mpsc::Sender<UserInput>, bridge_state: BridgeState) {
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

    let countdowns = app.global::<crate::CountdownsData>();
    let select_state = bridge_state.countdowns.clone();
    let select_activity = inputs.clone();
    let select_app = app.as_weak();
    countdowns.on_select(move |key| {
        select_activity.borrow_mut().activity();
        let display = select_state
            .lock()
            .expect("countdown bridge state poisoned")
            .select(key.as_str());
        if let (Some(display), Some(app)) = (display, select_app.upgrade()) {
            super::countdowns::apply(&app, &display);
        }
    });

    let detail_state = bridge_state.countdowns.clone();
    let detail_activity = inputs.clone();
    let detail_app = app.as_weak();
    countdowns.on_open_detail(move || {
        detail_activity.borrow_mut().activity();
        let display = detail_state
            .lock()
            .expect("countdown bridge state poisoned")
            .open_detail();
        if let (Some(display), Some(app)) = (display, detail_app.upgrade()) {
            super::countdowns::apply(&app, &display);
        }
    });

    let close_state = bridge_state.countdowns.clone();
    let close_activity = inputs.clone();
    let close_app = app.as_weak();
    countdowns.on_close_detail(move || {
        close_activity.borrow_mut().activity();
        let display = close_state
            .lock()
            .expect("countdown bridge state poisoned")
            .close_detail();
        if let (Some(display), Some(app)) = (display, close_app.upgrade()) {
            super::countdowns::apply(&app, &display);
        }
    });

    let remove_state = bridge_state.countdowns.clone();
    let remove_activity = inputs.clone();
    let remove_app = app.as_weak();
    countdowns.on_remove(move |key| {
        remove_activity.borrow_mut().activity();
        let key = key.to_string();
        let removal = remove_state
            .lock()
            .expect("countdown bridge state poisoned")
            .remove(&key);
        if let Some((display, id)) = removal {
            if let Some(app) = remove_app.upgrade() {
                super::countdowns::apply(&app, &display);
            }
            remove_activity
                .borrow()
                .send(UserInput::RemoveCountdown(id));

            let state = remove_state.clone();
            let app = remove_app.clone();
            slint::Timer::single_shot(Duration::from_millis(400), move || {
                let display = state
                    .lock()
                    .expect("countdown bridge state poisoned")
                    .finish_remove(&key);
                if let Some(app) = app.upgrade() {
                    super::countdowns::apply(&app, &display);
                }
            });
        }
    });

    let settings = app.global::<SettingsData>();
    let save = inputs.clone();
    settings.on_save_voice_url(move |url| {
        let mut input = save.borrow_mut();
        input.activity();
        input.send(UserInput::SaveVoiceUrl(url.to_string()));
    });
    let connect = inputs.clone();
    settings.on_connect_voice(move || {
        let mut input = connect.borrow_mut();
        input.activity();
        input.send(UserInput::VoiceConnect);
    });

    let timers = app.global::<TimersData>();
    let quick_start = inputs.clone();
    timers.on_quick_start(move |minutes| match u32::try_from(minutes) {
        Ok(minutes) => {
            let mut input = quick_start.borrow_mut();
            input.activity();
            input.send(UserInput::Timer(TimerInput::QuickStart(minutes)));
        }
        Err(error) => eprintln!("số phút hẹn giờ không hợp lệ {minutes}: {error}"),
    });
    let toggle_pause = inputs.clone();
    timers.on_toggle_pause(move |key| {
        if let Some(id) = parse_timer_id(key.as_str()) {
            let mut input = toggle_pause.borrow_mut();
            input.activity();
            input.send(UserInput::Timer(TimerInput::TogglePause(id)));
        }
    });
    let add_minute = inputs.clone();
    timers.on_add_minute(move |key| {
        if let Some(id) = parse_timer_id(key.as_str()) {
            let mut input = add_minute.borrow_mut();
            input.activity();
            input.send(UserInput::Timer(TimerInput::AddMinute(id)));
        }
    });
    let cancel = inputs.clone();
    timers.on_cancel(move |key| {
        if let Some(id) = parse_timer_id(key.as_str()) {
            let mut input = cancel.borrow_mut();
            input.activity();
            input.send(UserInput::Timer(TimerInput::Cancel(id)));
        }
    });

    let dismiss = inputs.clone();
    app.global::<AlertData>().on_dismiss(move || {
        let mut input = dismiss.borrow_mut();
        input.activity();
        input.send(UserInput::DismissAlert);
    });

    super::home::connect_inputs(app, inputs.clone());
    super::photos::connect_inputs(app, inputs.clone());
    super::center::connect_inputs(app, inputs);
}
