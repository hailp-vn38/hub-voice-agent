//! Trạng thái fixture của Lịch và nối callback hợp đồng.
use super::data::{self, Ev};
use crate::{App, CalDetail, CalendarData, PendingState};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

pub struct State {
    pub events: Vec<Ev>,
    pub sel: i32,
    pub next_month: bool,
    pub now_min: i32,
}

thread_local! {
    static STATE: Rc<RefCell<State>> = Rc::new(RefCell::new(State { events: data::seed(), sel: 0, next_month: false, now_min: 0 }));
}

pub fn state() -> Rc<RefCell<State>> {
    STATE.with(Rc::clone)
}

pub fn model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

/// Ghi lại toàn bộ model từ state.
pub fn refresh(app: &App) {
    let st = state();
    let st = st.borrow();
    let c = app.global::<CalendarData>();
    c.set_week(model(data::week_days(&st.events)));
    c.set_selected_day(data::day_key(st.sel).into());
    c.set_blocks(model(data::blocks(&st.events, st.sel, st.now_min)));
    c.set_now_min(st.now_min);
    c.set_upcoming(model(data::upcoming(&st.events, st.sel)));
    c.set_month(model(data::month(&st.events, st.next_month)));
}

pub fn init(app: &App) {
    let c = app.global::<CalendarData>();
    c.set_month_title("Tháng 11".into());
    c.set_synced("11:58".into());
    state().borrow_mut().now_min = minutes();
    refresh(app);

    let weak = app.as_weak();
    c.on_select_day(move |k| {
        let Some(app) = weak.upgrade() else { return };
        {
            let st = state();
            let mut st = st.borrow_mut();
            st.sel = data::parse_day(&k);
            st.events.retain(|e| e.pending != PendingState::Failed); // thẻ lỗi bị bỏ khi đổi ngày
        }
        app.global::<CalendarData>().set_month_view(false);
        refresh(&app);
    });
    let weak = app.as_weak();
    c.on_toggle_month(move || {
        if let Some(app) = weak.upgrade() {
            let c = app.global::<CalendarData>();
            c.set_month_view(!c.get_month_view());
        }
    });
    let weak = app.as_weak();
    c.on_open_detail(move |k| {
        let Some(app) = weak.upgrade() else { return };
        let st = state();
        let st = st.borrow();
        if let Some(e) = st.events.iter().find(|e| e.key == k.as_str()) {
            let time = if e.all_day { "Cả ngày".to_string() } else { format!("{} – {}", data::hm(e.start), data::hm(e.end)) };
            app.global::<CalendarData>().set_detail(CalDetail {
                visible: true, title: e.title.as_str().into(), time: time.into(),
                location: e.location.as_str().into(), description: e.description.as_str().into(),
            });
        }
    });
    let weak = app.as_weak();
    c.on_close_detail(move || {
        if let Some(app) = weak.upgrade() {
            let c = app.global::<CalendarData>();
            let mut d = c.get_detail();
            d.visible = false;
            c.set_detail(d);
        }
    });
}

fn minutes() -> i32 {
    (crate::ui::demo::clock::now_day_phase() * 1440.0) as i32
}

/// Mỗi phút: dời vạch "bây giờ", cập nhật sự kiện đang diễn ra.
pub fn tick(app: &App) {
    let now = minutes();
    let changed = {
        let st = state();
        let mut st = st.borrow_mut();
        let changed = st.now_min != now;
        st.now_min = now;
        changed
    };
    if changed {
        refresh(app);
    }
}
