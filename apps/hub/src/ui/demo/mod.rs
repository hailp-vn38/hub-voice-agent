//! Adapter demo: ghi fixture vào các global hợp đồng, không cần hub-core.
mod clock;
mod fixtures;
mod levels;
mod qr;
mod scenarios;
mod settings;

use crate::{App, DemoPanel, Nav, StatusBar};
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{rc::Rc, time::Duration};

/// Giữ giá trị trả về cho tới khi app thoát (timer và cửa sổ panel).
pub struct Demo {
    _tick: Timer,
    _levels: Timer,
    _panel: DemoPanel,
}

pub fn start(app: &App) -> anyhow::Result<Demo> {
    app.set_demo_mode(true);
    let weak = app.as_weak();
    app.global::<Nav>().on_navigate(move |s| {
        if let Some(app) = weak.upgrade() {
            app.global::<Nav>().set_screen(s);
        }
    });
    let weak = app.as_weak();
    app.global::<crate::AmbientData>().on_wake(move || {
        if let Some(app) = weak.upgrade() {
            app.global::<Nav>().set_screen(crate::Screen::Center);
        }
    });
    fixtures::init(app);

    let tick = Timer::default();
    let weak = app.as_weak();
    tick.start(TimerMode::Repeated, Duration::from_secs(1), move || {
        if let Some(app) = weak.upgrade() {
            app.set_clock(clock::now_hms().into());
            let bar = app.global::<StatusBar>();
            bar.set_time(clock::now_hms()[..5].into());
            bar.set_day_phase(clock::now_day_phase());
            fixtures::tick(&app);
        }
    });

    let panel = DemoPanel::new()?;
    let scenarios = scenarios::all();
    let names: Vec<SharedString> = scenarios.iter().map(|(n, _)| (*n).into()).collect();
    panel.set_names(ModelRc::from(Rc::new(VecModel::from(names))));
    let weak = app.as_weak();
    panel.on_run(move |i| {
        if let (Some(app), Some((_, f))) = (weak.upgrade(), scenarios.get(i as usize)) {
            f(&app);
        }
    });
    panel.show()?;
    Ok(Demo { _tick: tick, _levels: levels::start(app), _panel: panel })
}
