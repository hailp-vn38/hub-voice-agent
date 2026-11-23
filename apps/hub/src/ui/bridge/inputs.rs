use super::BridgeState;
use crate::{App, Nav, SettingsData, Voice};
use hub_core::{Screen as CoreScreen, UserInput};
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
        save.borrow().send(UserInput::SaveVoiceUrl(url.to_string()));
    });
    let connect = inputs;
    settings.on_connect_voice(move || {
        connect.borrow().send(UserInput::VoiceConnect);
    });
}
