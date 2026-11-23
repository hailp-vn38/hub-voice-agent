//! Kịch bản demo Hẹn giờ (ticket 11).
use super::state::{egg_or_first, remove, start, with_sim, TIMERS};
use crate::ui::demo::scenarios::show;
use crate::ui::demo::clock::now_secs;
use crate::{App, Screen, TimersData};
use slint::ComponentHandle;


pub fn boil_egg(app: &App) {
    show(app, Screen::Timers);
    start(app, Some("egg"), "Luộc trứng", 420);
}

pub fn add_another(app: &App) {
    show(app, Screen::Timers);
    let n = TIMERS.with(|s| s.borrow().len());
    let (label, secs) = [("Nướng bánh", 900), ("Trà", 180), ("Mì", 300)][n.saturating_sub(1).min(2)];
    if n < 4 {
        start(app, None, label, secs);
    }
}

pub fn final_seconds(app: &App) {
    if let Some(k) = egg_or_first() {
        with_sim(app, &k, |s| s.remaining = s.remaining.clamp(1, 10));
    }
}

pub fn pause_resume(app: &App) {
    if let Some(k) = egg_or_first() {
        with_sim(app, &k, |s| s.paused = !s.paused);
    }
}

pub fn add_two_minutes(app: &App) {
    if let Some(k) = egg_or_first() {
        with_sim(app, &k, |s| {
            s.add_minute();
            s.add_minute();
        });
    }
}

pub fn cancel_egg(app: &App) {
    remove(app, "egg");
}

pub fn finish(app: &App) {
    if let Some(k) = egg_or_first() {
        with_sim(app, &k, |s| {
            s.remaining = 0;
            s.mark_finished(now_secs());
        });
    }
}

pub fn focus_egg(app: &App) {
    show(app, Screen::Timers);
    let t = app.global::<TimersData>();
    t.set_focus("egg".into());
    t.set_focus_serial(t.get_focus_serial() + 1);
}
