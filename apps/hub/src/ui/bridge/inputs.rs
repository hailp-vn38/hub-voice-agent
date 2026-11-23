use crate::{AlertData, App, HomeData, Nav, PhotosData, SettingsData, TimersData, Voice};
use hub_core::{Focus, Screen as CoreScreen, TimerInput, UserInput};
use slint::{ComponentHandle, Model};
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

fn parse_timer_id(key: &str) -> Option<i64> {
    match key.parse() {
        Ok(id) => Some(id),
        Err(error) => {
            eprintln!("id hẹn giờ không hợp lệ {key:?}: {error}");
            None
        }
    }
}

fn send_focus(inputs: &Rc<RefCell<Inputs>>, focus: Option<Focus>) {
    let mut inputs = inputs.borrow_mut();
    inputs.activity();
    inputs.send(UserInput::Focus(focus));
}

fn adjacent_photo(data: &PhotosData, direction: i32) -> Option<String> {
    let viewer = data.get_viewer();
    let len = viewer.row_count();
    let index = usize::try_from(data.get_viewer_index())
        .ok()
        .filter(|index| *index < len)?;
    let next = (index as i64 + i64::from(direction)).rem_euclid(len as i64) as usize;
    viewer.row_data(next).map(|photo| photo.key.to_string())
}

fn photo_pinned(data: &PhotosData, key: &str) -> Option<bool> {
    let photos = data.get_photos();
    (0..photos.row_count()).find_map(|row| {
        photos
            .row_data(row)
            .filter(|photo| photo.key == key)
            .map(|photo| photo.pinned)
    })
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
    let connect = inputs.clone();
    settings.on_connect_voice(move || {
        connect.borrow().send(UserInput::VoiceConnect);
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

    let home = app.global::<HomeData>();
    let toggle_home = inputs.clone();
    home.on_toggle(move |key| {
        let mut input = toggle_home.borrow_mut();
        input.activity();
        input.send(UserInput::ToggleHome(key.to_string()));
    });

    let photos = app.global::<PhotosData>();
    let open = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_open(move |key| {
        if let Some(app) = app_weak.upgrade() {
            super::photos::stop_auto_advance(&app);
        }
        send_focus(&open, Some(Focus::Photo(key.to_string())));
    });

    let next = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_next(move || {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(key) = adjacent_photo(&data, 1) {
                super::photos::stop_auto_advance(&app);
                send_focus(&next, Some(Focus::Photo(key)));
            }
        }
    });

    let previous = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_previous(move || {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(key) = adjacent_photo(&data, -1) {
                super::photos::stop_auto_advance(&app);
                send_focus(&previous, Some(Focus::Photo(key)));
            }
        }
    });

    let close = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_close_viewer(move || {
        if let Some(app) = app_weak.upgrade() {
            super::photos::stop_auto_advance(&app);
        }
        send_focus(&close, None);
    });

    let toggle_pin = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_toggle_pin(move |key| {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(pinned) = photo_pinned(&data, key.as_str()) {
                super::photos::stop_auto_advance(&app);
                let mut input = toggle_pin.borrow_mut();
                input.activity();
                input.send(UserInput::PinPhoto {
                    id: key.to_string(),
                    pinned: !pinned,
                });
            }
        }
    });

    let remove = inputs;
    let app_weak = app.as_weak();
    photos.on_remove(move |key| {
        if let Some(app) = app_weak.upgrade() {
            super::photos::stop_auto_advance(&app);
        }
        let mut input = remove.borrow_mut();
        input.activity();
        input.send(UserInput::RemovePhoto {
            id: key.to_string(),
        });
    });
}
