//! Fixture và kịch bản Reminder (ticket 13).
use crate::{App, ReminderGroup, ReminderItem, ReminderNext, ReminderPast, RemindersData, Spawn};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use std::{rc::Rc, time::Duration};

fn item(key: &str, time: &str, text: &str, repeat: &str) -> ReminderItem {
    ReminderItem {
        key: key.into(),
        time: time.into(),
        text: text.into(),
        repeat_text: repeat.into(),
        repeats: !repeat.is_empty(),
        fading: false,
    }
}

fn past(key: &str, time: &str, text: &str, missed: bool) -> ReminderPast {
    ReminderPast { key: key.into(), time: time.into(), text: text.into(), missed }
}

fn model<T: Clone + 'static>(v: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(v)))
}

fn group(label: &str, items: Vec<ReminderItem>) -> ReminderGroup {
    ReminderGroup { label: label.into(), items: model(items) }
}

fn set_next(app: &App, it: &ReminderItem, until: &str) {
    let d = app.global::<RemindersData>();
    d.set_has_next(true);
    d.set_next(ReminderNext {
        key: it.key.clone(),
        time: it.time.clone(),
        until: until.into(),
        text: it.text.clone(),
        repeat_text: if it.repeats { "lặp hằng ngày".into() } else { "".into() },
    });
}

fn gym() -> ReminderItem {
    item("r-gym", "07:00", "Tập thể dục", "hằng ngày")
}

fn base(app: &App) {
    let d = app.global::<RemindersData>();
    let today = vec![item("r-med", "21:00", "Uống thuốc", ""), item("r-call", "21:30", "Gọi điện cho mẹ", "")];
    set_next(app, &today[0], "còn 25 phút");
    d.set_groups(model(vec![group("Hôm nay", today), group("Ngày mai", vec![gym()])]));
    d.set_title("3 sắp tới".into());
    d.set_recent(model(vec![past("p1", "18:00", "Đón con", false), past("p2", "19:30", "Tưới cây", true)]));
    d.set_focus(model(vec![]));
}

pub fn init(app: &App) {
    base(app);
}

pub fn empty(app: &App) {
    let d = app.global::<RemindersData>();
    d.set_has_next(false);
    d.set_groups(model(vec![]));
    d.set_recent(model(vec![]));
    d.set_title("0 sắp tới".into());
}

fn spawn(app: &App, key: &str) {
    let s = app.global::<Spawn>();
    s.set_key(key.into());
    s.set_serial(s.get_serial() + 1);
}

/// "Nhắc uống thuốc 21:00": mục mới bay từ Orb vào dòng thời gian.
pub fn add_medicine(app: &App) {
    base(app);
    let d = app.global::<RemindersData>();
    let today = vec![item("r-med2", "21:00", "Uống thuốc", ""), item("r-call", "21:30", "Gọi điện cho mẹ", "")];
    d.set_groups(model(vec![group("Hôm nay", today), group("Ngày mai", vec![gym()])]));
    spawn(app, "r-med2");
}

/// "Mỗi sáng 7 giờ tập thể dục": mục có icon lặp.
pub fn add_gym(app: &App) {
    base(app);
    spawn(app, "r-gym");
}

/// "Tối nay có nhắc gì?": sáng các mục của hôm nay.
pub fn focus_tonight(app: &App) {
    base(app);
    let d = app.global::<RemindersData>();
    let keys: Vec<SharedString> = vec!["r-med".into(), "r-call".into()];
    d.set_focus(model(keys));
    d.set_focus_serial(d.get_focus_serial() + 1);
}

fn update(app: &App, key: &str, f: impl Fn(&mut ReminderItem)) {
    let g = app.global::<RemindersData>().get_groups();
    for gi in 0..g.row_count() {
        let grp = g.row_data(gi).unwrap();
        for ii in 0..grp.items.row_count() {
            let mut it = grp.items.row_data(ii).unwrap();
            if it.key == key {
                f(&mut it);
                grp.items.set_row_data(ii, it);
            }
        }
    }
}

/// "Huỷ nhắc uống thuốc": mờ dần rồi gỡ.
pub fn cancel_medicine(app: &App) {
    base(app);
    update(app, "r-med", |i| i.fading = true);
    let weak = app.as_weak();
    slint::Timer::single_shot(Duration::from_millis(600), move || {
        if let Some(app) = weak.upgrade() {
            let d = app.global::<RemindersData>();
            let rest = vec![item("r-call", "21:30", "Gọi điện cho mẹ", "")];
            set_next(&app, &rest[0], "còn 55 phút");
            d.set_groups(model(vec![group("Hôm nay", rest), group("Ngày mai", vec![gym()])]));
            d.set_title("2 sắp tới".into());
        }
    });
}

/// "Hoãn 10 phút": mục đổi giờ.
pub fn postpone(app: &App) {
    base(app);
    update(app, "r-med", |i| i.time = "21:10".into());
    let d = app.global::<RemindersData>();
    let mut n = d.get_next();
    n.time = "21:10".into();
    n.until = "còn 35 phút".into();
    d.set_next(n);
}

/// "Một mục đã lỡ".
pub fn missed(app: &App) {
    base(app);
    let d = app.global::<RemindersData>();
    d.set_recent(model(vec![
        past("p1", "18:00", "Đón con", false),
        past("p2", "19:30", "Tưới cây", true),
        past("p3", "20:00", "Uống vitamin", true),
    ]));
}
