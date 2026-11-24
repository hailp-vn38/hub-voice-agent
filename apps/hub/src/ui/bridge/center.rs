use crate::{
    App, Center as CenterData, CenterCard, CenterCountdown, CenterEvent, CenterReminder,
    CenterTimer, CenterWeather, HomeKind, HomeTile, Menu,
};
use hub_core::{
    UserInput,
    present::{AmbientView, CenterCard as CoreCenterCard, CenterView, MenuView},
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

#[cfg(test)]
#[path = "center_tests.rs"]
mod tests;

use super::inputs::{Inputs, to_core_screen};

pub(super) fn apply_center(app: &App, center: &CenterView) {
    let data = app.global::<CenterData>();
    data.set_time(center.time.as_str().into());
    data.set_date(center.date.as_str().into());
    data.set_weather(CenterWeather {
        icon: center.weather.icon.as_str().into(),
        temp: center.weather.temp.as_str().into(),
        summary: center.weather.summary.as_str().into(),
        updated: center.weather.updated.as_str().into(),
        stale: center.weather.stale,
    });
    data.set_today(model(
        center
            .today
            .iter()
            .map(|event| CenterEvent {
                key: event.key.as_str().into(),
                time: event.time.as_str().into(),
                title: event.title.as_str().into(),
                now: event.now,
            })
            .collect(),
    ));
    data.set_reminders(model(
        center
            .reminders
            .iter()
            .map(|reminder| CenterReminder {
                key: reminder.key.as_str().into(),
                time: reminder.time.as_str().into(),
                text: reminder.text.as_str().into(),
            })
            .collect(),
    ));
    data.set_timer(CenterTimer {
        visible: center.timer.visible,
        key: center.timer.key.as_str().into(),
        label: center.timer.label.as_str().into(),
        remaining: center.timer.remaining.as_str().into(),
        progress: center.timer.progress,
    });
    data.set_countdown(CenterCountdown {
        visible: center.countdown.visible,
        key: center.countdown.key.as_str().into(),
        label: center.countdown.label.as_str().into(),
        value: center.countdown.value.as_str().into(),
        today: center.countdown.today,
    });
    data.set_home(model(
        center
            .home
            .iter()
            .map(|tile| HomeTile {
                key: tile.key.as_str().into(),
                name: tile.name.as_str().into(),
                kind: match tile.kind {
                    hub_core::present::HomeKind::Light => HomeKind::Light,
                    hub_core::present::HomeKind::Switch => HomeKind::Switch,
                    hub_core::present::HomeKind::Sensor => HomeKind::Sensor,
                },
                on: tile.on,
                value: tile.value.as_str().into(),
                pending: tile.pending,
                stale: tile.stale,
            })
            .collect(),
    ));
    data.set_home_safe(center.home_safe);
    data.set_focus(match center.focus {
        CoreCenterCard::None => CenterCard::None,
        CoreCenterCard::Weather => CenterCard::Weather,
        CoreCenterCard::Calendar => CenterCard::Calendar,
        CoreCenterCard::Reminders => CenterCard::Reminders,
        CoreCenterCard::Countdown => CenterCard::Countdown,
        CoreCenterCard::Home => CenterCard::Home,
        CoreCenterCard::Timer => CenterCard::Timer,
    });
    data.set_focus_serial(center.focus_serial as i32);
}

pub(super) fn apply_menu(app: &App, menu: &MenuView) {
    let data = app.global::<Menu>();
    data.set_weather(menu.weather.as_str().into());
    data.set_calendar(menu.calendar.as_str().into());
    data.set_home(menu.home.as_str().into());
    data.set_timers(menu.timers.as_str().into());
    data.set_countdowns(menu.countdowns.as_str().into());
    data.set_reminders(menu.reminders.as_str().into());
    data.set_photos(menu.photos.as_str().into());
    data.set_settings(menu.settings.as_str().into());
}

pub(super) fn apply_ambient(app: &App, ambient: &AmbientView) {
    let data = app.global::<crate::AmbientData>();
    data.set_time(ambient.time.as_str().into());
    data.set_temp(ambient.temp.as_str().into());
    data.set_icon(ambient.icon.as_str().into());
    data.set_night(ambient.night);
}

pub(super) fn connect_inputs(app: &App, inputs: Rc<RefCell<Inputs>>) {
    let center = app.global::<CenterData>();
    let open = inputs.clone();
    center.on_open(move |screen| {
        if let Some(screen) = to_core_screen(screen) {
            let mut inputs = open.borrow_mut();
            inputs.activity();
            inputs.send(UserInput::Open(screen));
        } else {
            eprintln!("hub-core không có màn first-boot; bỏ qua điều hướng");
        }
    });

    let toggle_home = inputs.clone();
    center.on_toggle_home(move |key| {
        let mut inputs = toggle_home.borrow_mut();
        inputs.activity();
        inputs.send(UserInput::ToggleHome(key.to_string()));
    });

    let wake = inputs;
    app.global::<crate::AmbientData>()
        .on_wake(move || wake.borrow_mut().activity());
}

fn model<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}
