//! Trạng thái hẹn giờ mô phỏng và đồng bộ vào TimersData.
use super::sim::{format_remaining, Phase, Sim};
use crate::ui::demo::clock::now_secs;
use crate::{App, TimerCard, TimerPhase, TimersData};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

thread_local! {
    pub(super) static TIMERS: RefCell<Vec<Sim>> = const { RefCell::new(Vec::new()) };
    static CARDS: Rc<VecModel<TimerCard>> = Rc::new(VecModel::default());
    static NEXT_ID: RefCell<u32> = const { RefCell::new(0) };
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

/// Đẩy TIMERS vào model theo thứ tự; cập nhật tại chỗ để thẻ không bị tạo lại.
fn sync(app: &App) {
    let now = now_secs();
    CARDS.with(|m| {
        TIMERS.with(|sims| {
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
    let running = TIMERS.with(|s| s.borrow().iter().filter(|t| t.phase() != Phase::Finished).count());
    let title = if running == 0 { "‹ Hẹn giờ".to_string() } else { format!("‹ Hẹn giờ · {running} đang chạy") };
    app.global::<TimersData>().set_title(title.into());
}

pub(super) fn start(app: &App, key: Option<&str>, label: &str, secs: u32) {
    let key = key.map(String::from).unwrap_or_else(|| {
        NEXT_ID.with(|n| {
            *n.borrow_mut() += 1;
            format!("t{}", n.borrow())
        })
    });
    if TIMERS.with(|s| s.borrow().iter().any(|t| t.key == key)) {
        return;
    }
    TIMERS.with(|s| s.borrow_mut().push(Sim::new(&key, label, secs)));
    crate::ui::demo::fixtures::spawn::from_orb(app, &key);
    sync(app);
}

pub(super) fn with_sim(app: &App, key: &str, f: impl FnOnce(&mut Sim)) {
    TIMERS.with(|s| s.borrow_mut().iter_mut().find(|t| t.key == key).map(f));
    sync(app);
}

/// Thẻ "trứng" nếu có, không thì thẻ đầu tiên.
pub(super) fn egg_or_first() -> Option<String> {
    TIMERS.with(|s| {
        let s = s.borrow();
        s.iter().find(|t| t.key == "egg").or(s.first()).map(|t| t.key.clone())
    })
}

pub fn init(app: &App) {
    let t = app.global::<TimersData>();
    t.set_timers(ModelRc::from(CARDS.with(|m| m.clone())));
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

pub(super) fn remove(app: &App, key: &str) {
    TIMERS.with(|s| s.borrow_mut().retain(|t| t.key != key));
    sync(app);
}

/// Setting → Dữ liệu → "Xoá hẹn giờ".
pub fn clear_all(app: &App) {
    TIMERS.with(|s| s.borrow_mut().clear());
    sync(app);
}

pub fn tick(app: &App) {
    let now = now_secs();
    TIMERS.with(|s| {
        for t in s.borrow_mut().iter_mut() {
            t.tick();
            t.mark_finished(now);
        }
    });
    sync(app);
}
