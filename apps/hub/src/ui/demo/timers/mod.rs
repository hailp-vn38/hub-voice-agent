//! Demo Hẹn giờ (ticket 11): trạng thái mô phỏng, tự đếm ngược mỗi giây.
mod sim;

use self::sim::{format_remaining, Phase, Sim};
use super::clock::now_secs;
use crate::{App, Nav, Screen, Spawn, TimerCard, TimerPhase, TimersData};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

thread_local! {
    static SIMS: RefCell<Vec<Sim>> = const { RefCell::new(Vec::new()) };
    static MODEL: Rc<VecModel<TimerCard>> = Rc::new(VecModel::default());
    static NEXT: RefCell<u32> = const { RefCell::new(0) };
}

fn card(s: &Sim, now: u64) -> TimerCard {
    TimerCard {
        key: s.key.as_str().into(),
        label: s.label.as_str().into(),
        remaining: format_remaining(s.remaining).into(),
        detail: s.detail(now).into(),
        progress: s.progress(),
        phase: match s.phase() {
            Phase::Running => TimerPhase::Running,
            Phase::Paused => TimerPhase::Paused,
            Phase::FinalSeconds => TimerPhase::FinalSeconds,
            Phase::Finished => TimerPhase::Finished,
        },
    }
}

/// Đẩy SIMS vào model theo thứ tự; cập nhật tại chỗ để thẻ không bị tạo lại.
fn sync(app: &App) {
    let now = now_secs();
    MODEL.with(|m| {
        SIMS.with(|sims| {
            let sims = sims.borrow();
            // Bỏ đúng hàng đã xoá (không dồn chỉ số) để thẻ còn lại giữ nguyên trạng thái.
            let mut i = 0;
            while i < m.row_count() {
                if m.row_data(i).is_some_and(|c| sims.iter().all(|s| s.key.as_str() != c.key.as_str())) {
                    m.remove(i);
                } else {
                    i += 1;
                }
            }
            for (i, s) in sims.iter().enumerate() {
                let c = card(s, now);
                if i < m.row_count() {
                    m.set_row_data(i, c);
                } else {
                    m.push(c);
                }
            }
        });
    });
    let running = SIMS.with(|s| s.borrow().iter().filter(|t| t.phase() != Phase::Finished).count());
    let title = if running == 0 { "‹ Hẹn giờ".to_string() } else { format!("‹ Hẹn giờ · {running} đang chạy") };
    app.global::<TimersData>().set_title(title.into());
}

fn start(app: &App, key: Option<&str>, label: &str, secs: u32) {
    let key = key.map(String::from).unwrap_or_else(|| {
        NEXT.with(|n| {
            *n.borrow_mut() += 1;
            format!("t{}", n.borrow())
        })
    });
    if SIMS.with(|s| s.borrow().iter().any(|t| t.key == key)) {
        return;
    }
    SIMS.with(|s| s.borrow_mut().push(Sim::new(&key, label, secs)));
    let sp = app.global::<Spawn>();
    sp.set_key(key.as_str().into());
    sp.set_serial(sp.get_serial() + 1);
    sync(app);
}

fn with_sim(app: &App, key: &str, f: impl FnOnce(&mut Sim)) {
    SIMS.with(|s| s.borrow_mut().iter_mut().find(|t| t.key == key).map(f));
    sync(app);
}

/// Thẻ "trứng" nếu có, không thì thẻ đầu tiên.
fn target() -> Option<String> {
    SIMS.with(|s| {
        let s = s.borrow();
        s.iter().find(|t| t.key == "egg").or(s.first()).map(|t| t.key.clone())
    })
}

pub fn init(app: &App) {
    let t = app.global::<TimersData>();
    t.set_timers(ModelRc::from(MODEL.with(|m| m.clone())));
    let weak = app.as_weak();
    t.on_quick_start(move |m| {
        if let Some(app) = weak.upgrade() {
            start(&app, None, &format!("{m} phút"), m as u32 * 60);
        }
    });
    let weak = app.as_weak();
    t.on_toggle_pause(move |k| {
        if let Some(app) = weak.upgrade() {
            with_sim(&app, &k, |s| s.paused = !s.paused);
        }
    });
    let weak = app.as_weak();
    t.on_add_minute(move |k| {
        if let Some(app) = weak.upgrade() {
            with_sim(&app, &k, Sim::add_minute);
        }
    });
    let weak = app.as_weak();
    t.on_cancel(move |k| {
        if let Some(app) = weak.upgrade() {
            remove(&app, &k);
        }
    });
    sync(app);
}

fn remove(app: &App, key: &str) {
    SIMS.with(|s| s.borrow_mut().retain(|t| t.key != key));
    sync(app);
}

pub fn tick(app: &App) {
    let now = now_secs();
    SIMS.with(|s| {
        for t in s.borrow_mut().iter_mut() {
            t.tick();
            t.mark_finished(now);
        }
    });
    sync(app);
}

fn show(app: &App) {
    app.set_show_sample(false);
    app.global::<Nav>().set_screen(Screen::Timers);
}

pub fn boil_egg(app: &App) {
    show(app);
    start(app, Some("egg"), "Luộc trứng", 420);
}

pub fn add_another(app: &App) {
    show(app);
    let n = SIMS.with(|s| s.borrow().len());
    let (label, secs) = [("Nướng bánh", 900), ("Trà", 180), ("Mì", 300)][n.saturating_sub(1).min(2)];
    if n < 4 {
        start(app, None, label, secs);
    }
}

pub fn final_seconds(app: &App) {
    if let Some(k) = target() {
        with_sim(app, &k, |s| s.remaining = s.remaining.min(10).max(1));
    }
}

pub fn pause_resume(app: &App) {
    if let Some(k) = target() {
        with_sim(app, &k, |s| s.paused = !s.paused);
    }
}

pub fn add_two_minutes(app: &App) {
    if let Some(k) = target() {
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
    if let Some(k) = target() {
        with_sim(app, &k, |s| {
            s.remaining = 0;
            s.mark_finished(now_secs());
        });
    }
}

pub fn focus_egg(app: &App) {
    show(app);
    let t = app.global::<TimersData>();
    t.set_focus("egg".into());
    t.set_focus_serial(t.get_focus_serial() + 1);
}
