//! Bảng kịch bản của Demo panel. Ticket sau đăng ký kịch bản bằng một dòng.
use crate::{App, Nav, Screen, StatusBar, Theme, Voice, VoiceState};
use slint::ComponentHandle;

pub type Scenario = (&'static str, fn(&App));

pub fn all() -> Vec<Scenario> {
    vec![
        ("Mở menu", |app| {
            app.set_show_sample(false);
            app.global::<Nav>().set_screen(Screen::Menu)
        }),
        ("Về trung tâm", |app| {
            app.set_show_sample(false);
            app.global::<Nav>().set_screen(Screen::Center)
        }),
        ("Mở Thời tiết (tool)", |app| {
            app.set_show_sample(false);
            app.global::<Nav>().set_screen(Screen::Weather)
        }),
        ("Tự về trung tâm", |app| app.global::<Nav>().set_screen(Screen::Center)),
        ("Offline bật/tắt", |app| {
            let bar = app.global::<StatusBar>();
            bar.set_offline(!bar.get_offline());
        }),
        ("Bảng mẫu", |app| app.set_show_sample(true)),
        ("Wake", |app| {
            let v = app.global::<Voice>();
            v.set_wake_serial(v.get_wake_serial() + 1);
        }),
        ("Nghe", |app| app.global::<Voice>().set_state(VoiceState::Listening)),
        ("Nghĩ", |app| app.global::<Voice>().set_state(VoiceState::Thinking)),
        ("Nói", |app| app.global::<Voice>().set_state(VoiceState::Speaking)),
        ("Tool chạy", |app| {
            let v = app.global::<Voice>();
            v.set_tool_serial(v.get_tool_serial() + 1);
        }),
        ("Lỗi", |app| {
            let v = app.global::<Voice>();
            v.set_error_serial(v.get_error_serial() + 1);
        }),
        ("Offline", |app| app.global::<Voice>().set_state(VoiceState::Offline)),
        ("Tắt mic", |app| {
            let v = app.global::<Voice>();
            v.set_mic_muted(!v.get_mic_muted());
        }),
        ("Về Idle", |app| app.global::<Voice>().set_state(VoiceState::Idle)),
        ("Chữ nhỏ", |app| app.global::<Theme>().set_text_scale(0.85)),
        ("Chữ vừa", |app| app.global::<Theme>().set_text_scale(1.0)),
        ("Chữ lớn", |app| app.global::<Theme>().set_text_scale(1.25)),
    ]
}
