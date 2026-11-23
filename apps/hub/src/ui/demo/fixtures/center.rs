//! Fixture màn trung tâm (ticket 07).
use crate::{
    App, Center, CenterCard, CenterCountdown, CenterEvent, CenterReminder, CenterTimer, CenterWeather, HomeData, Nav,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::rc::Rc;

fn event(time: &str, title: &str, now: bool) -> CenterEvent {
    CenterEvent { key: title.into(), time: time.into(), title: title.into(), now }
}

fn timer(visible: bool) -> CenterTimer {
    CenterTimer { visible, key: "t1".into(), label: "Luộc trứng".into(), remaining: "04:30".into(), progress: 0.45 }
}

fn countdown(visible: bool, today: bool) -> CenterCountdown {
    CenterCountdown {
        visible,
        key: "c1".into(),
        label: "Tết".into(),
        value: if today { "Hôm nay".into() } else { "12 ngày".into() },
        today,
    }
}

pub fn init(app: &App) {
    let c = app.global::<Center>();
    c.set_time("14:32".into());
    c.set_date("Thứ Hai, 23/11".into());
    c.set_weather(CenterWeather {
        icon: "sun".into(),
        temp: "27°C".into(),
        summary: "Nắng nhẹ".into(),
        updated: "14:30".into(),
        stale: false,
    });
    let today = vec![event("09:00", "Họp nhóm", false), event("14:30", "Đón con", true), event("19:00", "Ăn tối", false)];
    c.set_today(ModelRc::from(Rc::new(VecModel::from(today))));
    let rem = vec![
        CenterReminder { key: "r1".into(), time: "16:00".into(), text: "Uống thuốc".into() },
        CenterReminder { key: "r2".into(), time: "18:30".into(), text: "Mua rau".into() },
    ];
    c.set_reminders(ModelRc::from(Rc::new(VecModel::from(rem))));
    c.set_timer(timer(true));
    c.set_countdown(countdown(true, false));
    let tiles = app.global::<HomeData>().get_tiles();
    c.set_home(tiles);
    let weak = app.as_weak();
    c.on_open(move |s| {
        if let Some(app) = weak.upgrade() {
            app.global::<Nav>().invoke_navigate(s);
        }
    });
    let weak = app.as_weak();
    c.on_toggle_home(move |k| {
        if let Some(app) = weak.upgrade() {
            app.global::<HomeData>().invoke_toggle(k);
        }
    });
}

pub fn tick(app: &App) {
    app.global::<Center>().set_time(super::super::clock::now_hms()[..5].into());
}

pub fn add_timer(app: &App) {
    app.global::<Center>().set_timer(timer(true));
}

pub fn end_timer(app: &App) {
    app.global::<Center>().set_timer(timer(false));
}

pub fn countdown_today(app: &App) {
    app.global::<Center>().set_countdown(countdown(true, true));
}

/// "Hỏi mai có mưa": card thời tiết phóng lớn rồi trả về.
pub fn focus_weather(app: &App) {
    let c = app.global::<Center>();
    c.set_focus(CenterCard::Weather);
    c.set_focus_serial(c.get_focus_serial() + 1);
}

/// "Offline": dữ liệu thời tiết cũ.
pub fn stale(app: &App) {
    let c = app.global::<Center>();
    let mut w = c.get_weather();
    w.stale = true;
    c.set_weather(w);
}
