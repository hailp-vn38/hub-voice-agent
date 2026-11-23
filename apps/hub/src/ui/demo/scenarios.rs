//! Bảng kịch bản của Demo panel. Ticket sau đăng ký kịch bản bằng một dòng.
use crate::{AmbientData, App, Nav, Screen, StatusBar, Theme};
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
        ("Vào Ambient", |app| {
            app.set_show_sample(false);
            let a = app.global::<AmbientData>();
            a.set_time("14:32".into());
            a.set_temp("27°C".into());
            a.set_night(false);
            app.global::<Nav>().set_screen(Screen::Ambient)
        }),
        ("Ambient ban đêm", |app| {
            let a = app.global::<AmbientData>();
            a.set_time("02:07".into());
            a.set_temp("24°C".into());
            a.set_night(true);
            app.global::<Nav>().set_screen(Screen::Ambient)
        }),
        ("Bảng mẫu", |app| app.set_show_sample(true)),
        ("Lần đầu: 3 bước", |app| super::fixtures::run_first_boot(app, false)),
        ("Lần đầu: sai mật khẩu", |app| super::fixtures::run_first_boot(app, true)),
        ("Chữ nhỏ", |app| app.global::<Theme>().set_text_scale(0.85)),
        ("Chữ vừa", |app| app.global::<Theme>().set_text_scale(1.0)),
        ("Chữ lớn", |app| app.global::<Theme>().set_text_scale(1.25)),
    ]
}
