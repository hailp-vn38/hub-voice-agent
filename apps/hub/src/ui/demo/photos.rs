//! Fixture và kịch bản demo cho màn Ảnh (ticket 15). Trạng thái ảnh giữ ở Rust, đẩy vào PhotosData.
use crate::{App, Nav, Photo, PhotoState, PhotosData, Screen, Spawn};
use slint::{ComponentHandle, Image, ModelRc, VecModel};
use std::{cell::RefCell, path::Path, rc::Rc, time::Duration};

#[cfg(test)]
#[path = "photos_tests.rs"]
mod tests;

/// Chỉ số kế tiếp, quay vòng; `delta` là +1 hoặc -1.
fn step(index: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    (index as i32 + delta).rem_euclid(len as i32) as usize
}

#[derive(Default)]
struct State {
    all: Vec<Photo>, // mới nhất trước
    viewer: Vec<String>,
    index: usize,
    last_sent: Vec<String>,
}

thread_local! { static STATE: Rc<RefCell<State>> = Rc::default(); }

fn state() -> Rc<RefCell<State>> {
    STATE.with(Rc::clone)
}

fn load(n: usize) -> Image {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("assets/demo/photo-{:02}.png", n % 10 + 1));
    Image::load_from_path(&path).unwrap_or_default()
}

fn photo(key: &str, n: usize, caption: &str, source: &str, time: &str, st: PhotoState) -> Photo {
    let ready = st == PhotoState::Ready;
    Photo {
        key: key.into(),
        thumb: if ready { load(n) } else { Image::default() },
        full: if ready { load(n) } else { Image::default() },
        caption: caption.into(),
        source: source.into(),
        time: time.into(),
        pinned: false,
        state: st,
    }
}

fn sync(app: &App) {
    let st = state();
    let s = st.borrow();
    let d = app.global::<PhotosData>();
    let pinned = s.all.iter().filter(|p| p.pinned).count();
    d.set_title(format!("Ảnh · {} ảnh", s.all.len()).into());
    d.set_pinned_text(if pinned > 0 { format!("{pinned} đã giữ").into() } else { "".into() });
    d.set_photos(ModelRc::from(Rc::new(VecModel::from(s.all.clone()))));
    let viewer: Vec<Photo> = s.viewer.iter().filter_map(|k| s.all.iter().find(|p| p.key == *k).cloned()).collect();
    d.set_viewer(ModelRc::from(Rc::new(VecModel::from(viewer))));
    d.set_viewer_index(s.index as i32);
}

fn show_viewer(app: &App, keys: Vec<String>, auto: bool) {
    {
        let st = state();
        let mut s = st.borrow_mut();
        s.viewer = keys;
        s.index = 0;
    }
    sync(app);
    let d = app.global::<PhotosData>();
    d.set_auto_advance(auto);
    d.set_viewing(true);
    app.set_show_sample(false);
    app.global::<Nav>().set_screen(Screen::Photos);
}

fn spawn(app: &App) {
    let s = app.global::<Spawn>();
    s.set_key("photo".into());
    s.set_serial(s.get_serial() + 1);
}

pub fn init(app: &App) {
    {
        let st = state();
        let mut s = st.borrow_mut();
        let places = ["Vịnh Hạ Long", "Hội An", "Sa Pa", "Phú Quốc", "Huế", "Đà Nẵng", "Ninh Bình", "Nha Trang", "Cần Thơ", "Mũi Né"];
        s.all = places
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let st = if i == 6 { PhotoState::Failed } else { PhotoState::Ready };
                photo(&format!("p{i}"), i, name, "vnexpress.net", &format!("{} giờ trước", i + 1), st)
            })
            .collect();
    }
    sync(app);
    let d = app.global::<PhotosData>();
    let w = app.as_weak();
    d.on_open(move |key| {
        if let Some(app) = w.upgrade() {
            show_viewer(&app, vec![key.to_string()], false);
        }
    });
    let w = app.as_weak();
    d.on_next(move || go(&w, 1));
    let w = app.as_weak();
    d.on_previous(move || go(&w, -1));
    let w = app.as_weak();
    d.on_close_viewer(move || {
        if let Some(app) = w.upgrade() {
            app.global::<PhotosData>().set_viewing(false);
        }
    });
    let w = app.as_weak();
    d.on_toggle_pin(move |key| {
        if let Some(app) = w.upgrade() {
            if let Some(p) = state().borrow_mut().all.iter_mut().find(|p| p.key == key) {
                p.pinned = !p.pinned;
            }
            sync(&app);
        }
    });
    let w = app.as_weak();
    d.on_remove(move |key| {
        if let Some(app) = w.upgrade() {
            let empty = {
                let st = state();
                let mut s = st.borrow_mut();
                s.all.retain(|p| p.key != key);
                s.viewer.retain(|k| k.as_str() != key.as_str());
                s.index = s.index.min(s.viewer.len().saturating_sub(1));
                s.viewer.is_empty()
            };
            if empty {
                app.global::<PhotosData>().set_viewing(false);
            }
            sync(&app);
        }
    });
}

fn go(w: &slint::Weak<App>, delta: i32) {
    if let Some(app) = w.upgrade() {
        {
            let st = state();
            let mut s = st.borrow_mut();
            s.index = step(s.index, s.viewer.len(), delta);
        }
        sync(&app);
    }
}

/// "Agent gửi 3 ảnh Đà Lạt": khung chờ, rồi có ảnh, tự chuyển.
pub fn agent_sends(app: &App) {
    let keys: Vec<String> = (0..3).map(|i| format!("dl{i}")).collect();
    {
        let st = state();
        let mut s = st.borrow_mut();
        s.all.retain(|p| !keys.iter().any(|k| k.as_str() == p.key.as_str()));
        for (i, k) in keys.iter().enumerate().rev() {
            s.all.insert(0, photo(k, i + 3, &format!("Đà Lạt {}", i + 1), "dalat.gov.vn", "Vừa xong", PhotoState::Loading));
        }
        s.last_sent = keys.clone();
    }
    show_viewer(app, keys.clone(), true);
    spawn(app);
    let w = app.as_weak();
    slint::Timer::single_shot(Duration::from_millis(1800), move || {
        let Some(app) = w.upgrade() else { return };
        for (i, k) in keys.iter().enumerate() {
            if let Some(p) = state().borrow_mut().all.iter_mut().find(|p| p.key == *k) {
                *p = photo(k, i + 3, &format!("Đà Lạt {}", i + 1), "dalat.gov.vn", "Vừa xong", PhotoState::Ready);
            }
        }
        sync(&app);
    });
}

pub fn failed(app: &App) {
    {
        let st = state();
        let mut s = st.borrow_mut();
        s.all.retain(|p| p.key != "bad");
        s.all.insert(0, photo("bad", 0, "Ảnh lỗi", "example.com", "Vừa xong", PhotoState::Failed));
    }
    show_viewer(app, vec!["bad".into()], false);
    spawn(app);
}

pub fn next(app: &App) {
    app.global::<PhotosData>().invoke_next();
}

pub fn previous(app: &App) {
    app.global::<PhotosData>().invoke_previous();
}

pub fn pin_current(app: &App) {
    let key = {
        let st = state();
        let s = st.borrow();
        s.viewer.get(s.index).cloned()
    };
    if let Some(k) = key {
        app.global::<PhotosData>().invoke_toggle_pin(k.into());
    }
}

pub fn reopen(app: &App) {
    let keys = {
        let st = state();
        let s = st.borrow();
        if s.last_sent.is_empty() { vec!["p0".to_string(), "p1".to_string(), "p2".to_string()] } else { s.last_sent.clone() }
    };
    show_viewer(app, keys, false);
}
