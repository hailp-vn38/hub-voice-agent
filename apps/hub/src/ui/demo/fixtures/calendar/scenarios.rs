//! Kịch bản demo của màn Lịch.
use super::data::{self, ev};
use super::state::{refresh, state};
use crate::ui::demo::scenarios::show;
use crate::{App, CalKind, CalendarData, PendingState, Screen};
use slint::{ComponentHandle, Model, Timer};
use std::time::Duration;

fn open(app: &App, sel: i32, focus: Option<String>) {
    show(app, Screen::Calendar);
    let c = app.global::<CalendarData>();
    c.set_stale(false);
    c.set_month_view(false);
    c.set_month_title("Tháng 11".into());
    {
        let st = state();
        let mut st = st.borrow_mut();
        st.sel = sel;
        st.next_month = false;
        st.events.retain(|e| e.key != "new");
    }
    refresh(app);
    if let Some(f) = focus {
        c.set_focus(f.into());
        c.set_focus_serial(c.get_focus_serial() + 1);
    }
}

/// Sự kiện kế tiếp trong ngày `sel` (theo giờ hiện tại nếu là hôm nay), nếu có.
fn next_in_day(sel: i32) -> Option<String> {
    let st = state();
    let st = st.borrow();
    let mut day: Vec<_> = st.events.iter().filter(|e| e.day == sel && !e.all_day).collect();
    day.sort_by_key(|e| e.start);
    let now = if sel == 0 { st.now_min } else { 0 };
    day.iter().find(|e| e.end > now).or(day.first()).map(|e| e.key.clone())
}

pub fn today(app: &App) {
    open(app, 0, next_in_day(0));
}

pub fn friday(app: &App) {
    open(app, 4, next_in_day(4));
}

/// "Lịch tiếp theo": sự kiện kế tiếp hôm nay, hoặc sự kiện đầu của ngày kế tiếp có lịch.
pub fn next(app: &App) {
    let (sel, key) = match next_in_day(0).filter(|k| {
        let st = state();
        let st = st.borrow();
        st.events.iter().any(|e| &e.key == k && e.end > st.now_min)
    }) {
        Some(k) => (0, Some(k)),
        None => (1, next_in_day(1)),
    };
    open(app, sel, key);
}

pub fn next_month(app: &App) {
    open(app, 0, None);
    state().borrow_mut().next_month = true;
    refresh(app);
    let c = app.global::<CalendarData>();
    c.set_month_title("Tháng 12".into());
    c.set_month_view(true);
}

fn create(app: &App, ok: bool) {
    open(app, 1, None);
    {
        let st = state();
        let mut st = st.borrow_mut();
        let mut e = ev("new", 1, CalKind::Event, "Khám răng", 15 * 60, 16 * 60);
        e.fresh = true;
        e.pending = PendingState::Saving;
        st.events.push(e);
    }
    refresh(app);
    crate::ui::demo::fixtures::spawn::from_orb(app, "new");
    let weak = app.as_weak();
    Timer::single_shot(Duration::from_millis(1500), move || {
        let Some(app) = weak.upgrade() else { return };
        let p = if ok { PendingState::Saved } else { PendingState::Failed };
        set_pending(&app, p);
    });
}

/// Đổi trạng thái chờ tại chỗ (không dựng lại model, tránh phát lại hiệu ứng sinh).
fn set_pending(app: &App, p: PendingState) {
    if let Some(e) = state().borrow_mut().events.iter_mut().find(|e| e.key == "new") {
        e.pending = p;
    }
    let c = app.global::<CalendarData>();
    let blocks = c.get_blocks();
    for i in 0..blocks.row_count() {
        let mut b = blocks.row_data(i).unwrap();
        if b.key == "new" {
            b.pending = p;
            blocks.set_row_data(i, b);
        }
    }
    let st = state();
    let st = st.borrow();
    c.set_week(super::state::model(data::week_days(&st.events)));
}

pub fn create_ok(app: &App) {
    create(app, true);
}

pub fn create_fail(app: &App) {
    create(app, false);
}

pub fn offline(app: &App) {
    open(app, 0, None);
    let c = app.global::<CalendarData>();
    c.set_stale(true);
    c.set_synced("09:12".into());
}
