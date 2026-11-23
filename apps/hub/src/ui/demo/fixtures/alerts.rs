//! Kịch bản demo cho lớp báo và thanh trượt giọng nói (ticket 14).
use crate::{AlertData, AlertKind, App, SliderKind, VoiceSlider};
use slint::ComponentHandle;

pub fn alert(app: &App, kind: AlertKind, title: &str, time: &str, body: &str) {
    let a = app.global::<AlertData>();
    a.set_kind(kind);
    a.set_title(title.into());
    a.set_time(time.into());
    a.set_body(body.into());
    a.set_serial(a.get_serial() + 1);
    let weak = app.as_weak();
    a.on_dismiss(move || {
        if let Some(app) = weak.upgrade() {
            app.global::<AlertData>().set_kind(AlertKind::None);
        }
    });
}

pub fn slider(app: &App, kind: SliderKind, value: f32) {
    let s = app.global::<VoiceSlider>();
    s.set_kind(kind);
    s.set_value(value);
    s.set_serial(s.get_serial() + 1);
}
