//! Bảng kịch bản của Demo panel. Ticket sau đăng ký kịch bản bằng một dòng.
use crate::{App, Nav, Screen};
use slint::ComponentHandle;

pub type Scenario = (&'static str, fn(&App));

pub fn all() -> Vec<Scenario> {
    vec![
        ("Mở menu", |app| app.global::<Nav>().set_screen(Screen::Menu)),
        ("Về trung tâm", |app| app.global::<Nav>().set_screen(Screen::Center)),
    ]
}
