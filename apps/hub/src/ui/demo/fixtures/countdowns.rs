//! Fixture và kịch bản demo cho màn Đếm ngược (ticket 12).
use crate::{App, Countdown, CountdownUnit, CountdownsData, Nav, Screen, Spawn};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;

thread_local! {
    static OTHERS: Rc<VecModel<Countdown>> = Rc::new(VecModel::default());
}

fn dec_hms(v: &str) -> String {
    let p: Vec<u32> = v.split(':').filter_map(|s| s.parse().ok()).collect();
    let [h, m, s] = p[..] else { return v.into() };
    let t = (h * 3600 + m * 60 + s).saturating_sub(1);
    format!("{:02}:{:02}:{:02}", t / 3600, t % 3600 / 60, t % 60)
}

fn dec_days(v: &str) -> String {
    v.parse::<u32>().map_or_else(|_| v.into(), |n| n.saturating_sub(1).to_string())
}

fn cd(key: &str, label: &str, value: &str, unit: CountdownUnit, target: &str, lunar: &str, repeat: bool, progress: f32) -> Countdown {
    Countdown {
        key: key.into(),
        label: label.into(),
        value: value.into(),
        unit,
        target: target.into(),
        lunar: lunar.into(),
        repeat_text: if repeat { "hằng năm" } else { "" }.into(),
        progress,
        progress_text: format!("{}% chặng đường", (progress * 100.0).round()).into(),
    }
}

fn trip() -> Countdown {
    cd("trip", "Chuyến đi Đà Lạt", "12", CountdownUnit::Days, "5/12/2026", "", false, 0.8)
}
fn birthday() -> Countdown {
    cd("bday", "Sinh nhật", "109", CountdownUnit::Days, "12/3/2027", "", true, 0.3)
}
fn tet() -> Countdown {
    cd("tet", "Tết Nguyên đán", "75", CountdownUnit::Days, "6/2/2027", "Mùng 1 tháng Giêng", true, 0.2)
}

pub fn init(app: &App) {
    let d = app.global::<CountdownsData>();
    d.set_title("Đếm ngược · 2 mốc".into());
    d.set_main(trip());
    d.set_has_main(true);
    OTHERS.with(|o| {
        o.set_vec(vec![birthday()]);
        d.set_others(ModelRc::from(o.clone()));
    });
    let weak = app.as_weak();
    d.on_select(move |key| {
        let Some(app) = weak.upgrade() else { return };
        let d = app.global::<CountdownsData>();
        OTHERS.with(|o| {
            if let Some(i) = o.iter().position(|c| c.key == key) {
                let old = d.get_main();
                d.set_main(o.row_data(i).unwrap());
                o.set_row_data(i, old);
            }
        });
    });
    let weak = app.as_weak();
    d.on_open_detail(move || {
        if let Some(app) = weak.upgrade() {
            app.global::<CountdownsData>().set_detail_open(true);
        }
    });
    let weak = app.as_weak();
    d.on_close_detail(move || {
        if let Some(app) = weak.upgrade() {
            app.global::<CountdownsData>().set_detail_open(false);
        }
    });
    let weak = app.as_weak();
    d.on_remove(move |key| {
        if let Some(app) = weak.upgrade() {
            remove(&app, &key);
        }
    });
}

fn refresh_title(app: &App) {
    let d = app.global::<CountdownsData>();
    let n = OTHERS.with(|o| o.row_count()) + usize::from(d.get_has_main());
    d.set_title(format!("Đếm ngược · {n} mốc").into());
}

fn remove(app: &App, key: &str) {
    let d = app.global::<CountdownsData>();
    d.set_detail_open(false);
    if d.get_has_main() && d.get_main().key == key {
        OTHERS.with(|o| {
            if o.row_count() > 0 {
                d.set_main(o.remove(0));
            } else {
                d.set_has_main(false);
            }
        });
    } else {
        OTHERS.with(|o| {
            if let Some(i) = o.iter().position(|c| c.key == key) {
                o.remove(i);
            }
        });
    }
    refresh_title(app);
}

/// Mỗi giây: mốc `live` đếm lùi.
pub fn tick(app: &App) {
    let d = app.global::<CountdownsData>();
    let mut m = d.get_main();
    if d.get_has_main() && m.unit == CountdownUnit::Live {
        m.value = dec_hms(&m.value).into();
        d.set_main(m);
    }
}

fn open(app: &App) {
    app.set_show_sample(false);
    app.global::<Nav>().set_screen(Screen::Countdowns);
}

fn spawn(app: &App, key: &str) {
    let s = app.global::<Spawn>();
    s.set_key(key.into());
    s.set_serial(s.get_serial() + 1);
}

/// "Đếm ngược đến Tết": thành mốc chính, mốc cũ trượt sang danh sách.
pub fn add_tet(app: &App) {
    open(app);
    let d = app.global::<CountdownsData>();
    if d.get_has_main() && d.get_main().key == "tet" {
        return;
    }
    if d.get_has_main() {
        OTHERS.with(|o| {
            if let Some(i) = o.iter().position(|c| c.key == "tet") {
                o.remove(i);
            }
            o.insert(0, d.get_main());
        });
    }
    d.set_main(tet());
    d.set_has_main(true);
    refresh_title(app);
    spawn(app, "tet");
}

/// "Sinh nhật 12/3": vào danh sách.
pub fn add_birthday(app: &App) {
    open(app);
    let has = OTHERS.with(|o| o.iter().any(|c| c.key == "bday"));
    if !has {
        OTHERS.with(|o| o.push(birthday()));
        refresh_title(app);
    }
    spawn(app, "bday");
}

/// "Qua nửa đêm": số ngày giảm một, kích hoạt flip.
pub fn midnight(app: &App) {
    open(app);
    let d = app.global::<CountdownsData>();
    let mut m = d.get_main();
    m.value = dec_days(&m.value).into();
    d.set_main(m);
}

/// "Còn < 24 giờ": HH:MM:SS chạy trực tiếp.
pub fn live(app: &App) {
    open(app);
    let d = app.global::<CountdownsData>();
    let mut m = d.get_main();
    m.unit = CountdownUnit::Live;
    m.value = "05:42:10".into();
    d.set_main(m);
}

/// "Hôm nay là Tết": pháo giấy từ Orb.
pub fn today(app: &App) {
    open(app);
    let d = app.global::<CountdownsData>();
    let mut m = d.get_main();
    m.unit = CountdownUnit::Today;
    m.progress = 1.0;
    m.progress_text = "100% chặng đường".into();
    d.set_main(m);
    d.set_confetti_serial(d.get_confetti_serial() + 1);
}

/// "Chuyến đi đã qua".
pub fn passed(app: &App) {
    open(app);
    let d = app.global::<CountdownsData>();
    let mut m = d.get_main();
    m.unit = CountdownUnit::Passed;
    d.set_main(m);
}

/// "Còn bao nhiêu ngày đến Tết?": làm nổi mốc Tết.
pub fn focus_tet(app: &App) {
    add_tet(app);
    let d = app.global::<CountdownsData>();
    d.set_focus("tet".into());
    d.set_focus_serial(d.get_focus_serial() + 1);
}

pub fn remove_trip(app: &App) {
    open(app);
    remove(app, "trip");
}

#[cfg(test)]
#[path = "countdowns_tests.rs"]
mod tests;
