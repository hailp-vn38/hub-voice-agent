//! Fixture màn lần đầu khởi động: ba bước nối tiếp và lỗi sai mật khẩu.
use crate::ui::demo::scenarios::show;
use crate::{App, BootStep, FirstBootData, Screen, ui::demo::qr};
use slint::{ComponentHandle, Timer, TimerMode};
use std::{cell::RefCell, time::Duration};

thread_local! { static STEPS: RefCell<Timer> = RefCell::new(Timer::default()); }

fn setup(app: &App, step: BootStep, url: &str, home: &str, error: &str) {
    let d = app.global::<FirstBootData>();
    d.set_ap_name("AI-Hub-Setup".into());
    d.set_ap_qr(qr::image("WIFI:T:nopass;S:AI-Hub-Setup;;"));
    d.set_setup_url(url.into());
    d.set_setup_qr(qr::image(&format!("http://{url}")));
    d.set_home_network(home.into());
    d.set_error(error.into());
    d.set_step(step);
}

pub fn init(app: &App) {
    let weak = app.as_weak();
    // Demo: "Xong" đưa về màn trung tâm (thật: hub-core quyết màn kế tiếp).
    app.global::<FirstBootData>().on_done(move || {
        if let Some(app) = weak.upgrade() {
            show(&app, Screen::Center);
        }
    });
}

/// Chạy kịch bản: bước 1 → 2 → 3, hoặc sai mật khẩu rồi về bước 1 kèm lỗi.
pub fn run(app: &App, wrong_password: bool) {
    show(app, Screen::FirstBoot);
    setup(app, BootStep::AccessPoint, "192.168.4.1", "", "");
    let weak = app.as_weak();
    let mut n = 0;
    STEPS.with(|t| {
        t.borrow().start(TimerMode::Repeated, Duration::from_secs(3), move || {
            let Some(app) = weak.upgrade() else { return };
            n += 1;
            match (n, wrong_password) {
                (1, _) => setup(&app, BootStep::Joining, "192.168.4.1", "Nha-Minh", ""),
                (2, false) => setup(&app, BootStep::Joined, "192.168.1.42", "Nha-Minh", ""),
                (2, true) => setup(&app, BootStep::AccessPoint, "192.168.4.1", "", "Sai mật khẩu Wi-Fi. Thử lại."),
                _ => STEPS.with(|t| t.borrow().stop()),
            }
        });
    });
}
