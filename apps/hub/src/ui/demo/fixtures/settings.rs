//! Fixture và hành vi demo của Setting: lựa chọn áp lại vào fixture.
use crate::{
    App, AudioSettings, ConnectionSettings, DisplaySettings, HomeEntity, HomeSettings, SettingsData,
    SystemSettings, Theme, VoiceSettingsView,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::{cell::RefCell, time::Duration};

thread_local! {
    static RES_TIMER: Timer = Timer::default();
    static PREV_RES: RefCell<Option<SharedString>> = const { RefCell::new(None) };
}

fn strs(v: &[&str]) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(v.iter().map(|s| SharedString::from(*s)).collect::<Vec<_>>()))
}

/// Cỡ chữ theo nhãn; None nếu nhãn lạ.
fn text_scale(label: &str) -> Option<f32> {
    match label {
        "Nhỏ" => Some(0.85),
        "Vừa" => Some(1.0),
        "Lớn" => Some(1.25),
        _ => None,
    }
}

pub fn init(app: &App) {
    let s = app.global::<SettingsData>();
    s.set_display(DisplaySettings {
        brightness: 0.7,
        auto_brightness: "Tắt".into(),
        resolution: "1920x1080".into(),
        resolutions: strs(&["1920x1080", "1280x720"]),
        text_size: "Vừa".into(),
        ambient_after: "5 phút".into(),
        ambient_night: true,
        ambient_night_range: "22:00-06:00".into(),
        ambient_night_ranges: strs(&["21:00-06:00", "22:00-06:00", "23:00-07:00"]),
    });
    s.set_audio(AudioSettings {
        volume: 0.5,
        ui_sounds: true,
        timer_sound: "Chuông".into(),
        reminder_sound: "Nhẹ".into(),
        sounds: strs(&["Chuông", "Nhẹ", "Gõ"]),
    });
    s.set_voice(VoiceSettingsView { wake_word: true, sensitivity: "Vừa".into(), mic_muted: false, read_reminder: true });
    let entities = ["Đèn phòng khách", "Quạt", "Điều hoà", "Rèm", "Cửa", "Máy lọc khí"]
        .iter()
        .enumerate()
        .map(|(i, n)| HomeEntity { name: (*n).into(), selected: i < 3 })
        .collect::<Vec<_>>();
    s.set_home(HomeSettings { entities: ModelRc::new(VecModel::from(entities)) });
    s.set_connection(ConnectionSettings {
        server_status: "Đã kết nối · 42 ms".into(),
        device_id: "demo-0001".into(),
        error: "".into(),
        enrolled: true,
        enrollment_code: "".into(),
        wifi_name: "Nha-5G".into(),
        wifi_strength: 3,
        wifi_ip: "192.168.1.20".into(),
        ha_status: "Lần lấy gần nhất 09:41".into(),
        calendar_status: "Lần lấy gần nhất 09:40".into(),
        weather_status: "Lỗi gần nhất: hết thời gian (09:10)".into(),
    });
    s.set_system(SystemSettings {
        version: "0.1.0-demo".into(),
        timezone: "Asia/Ho_Chi_Minh".into(),
        timezones: strs(&["Asia/Ho_Chi_Minh", "Asia/Bangkok", "Asia/Tokyo", "UTC"]),
    });
    s.set_setup_url("192.168.1.20:8080".into());
    s.set_setup_qr(crate::ui::demo::qr::image("http://192.168.1.20:8080"));

    let weak = app.as_weak();
    s.on_set_brightness(move |v| {
        if let Some(app) = weak.upgrade() {
            let g = app.global::<SettingsData>();
            g.set_display(DisplaySettings { brightness: v, ..g.get_display() });
        }
    });
    let weak = app.as_weak();
    s.on_set_volume(move |v| {
        if let Some(app) = weak.upgrade() {
            let g = app.global::<SettingsData>();
            g.set_audio(AudioSettings { volume: v, ..g.get_audio() });
        }
    });
    let weak = app.as_weak();
    s.on_choose(move |key, val| {
        if let Some(app) = weak.upgrade() {
            choose(&app, &key, val);
        }
    });
    let weak = app.as_weak();
    s.on_toggle(move |key, on| {
        if let Some(app) = weak.upgrade() {
            toggle(&app, &key, on);
        }
    });
    // Demo: không có âm thanh/dịch vụ thật nên chỉ nhận callback.
    s.on_preview_sound(|_| {});
    s.on_system_action(|_| {});
    let weak = app.as_weak();
    s.on_clear(move |key| {
        if let Some(app) = weak.upgrade() {
            let g = app.global::<SettingsData>();
            let mut c = g.get_connection();
            match key.as_str() {
                "cache" => {
                    c.calendar_status = "Đã xoá cache".into();
                    c.weather_status = "Đã xoá cache".into();
                }
                "photos" => super::photos::clear_all(&app),
                "timers" => super::timers::clear_all(&app),
                "countdowns" => super::countdowns::clear_all(&app),
                "reminders" => super::reminders::empty(&app),
                _ => {}
            }
            g.set_connection(c);
        }
    });
    let weak = app.as_weak();
    s.on_keep_resolution(move || {
        RES_TIMER.with(|t| t.stop());
        if let Some(app) = weak.upgrade() {
            app.global::<SettingsData>().set_resolution_confirm(0);
        }
    });
}

fn choose(app: &App, key: &str, val: SharedString) {
    let g = app.global::<SettingsData>();
    let mut d = g.get_display();
    let mut a = g.get_audio();
    match key {
        "auto-brightness" => d.auto_brightness = val,
        "text-size" => {
            if let Some(x) = text_scale(&val) {
                app.global::<Theme>().set_text_scale(x);
            }
            d.text_size = val;
        }
        "ambient-after" => d.ambient_after = val,
        "ambient-night-range" => d.ambient_night_range = val,
        "resolution" => {
            if d.resolution == val {
                return;
            }
            PREV_RES.with(|p| *p.borrow_mut() = Some(d.resolution.clone()));
            d.resolution = val;
            start_resolution_countdown(app);
        }
        "timer-sound" => a.timer_sound = val,
        "reminder-sound" => a.reminder_sound = val,
        "timezone" => g.set_system(SystemSettings { timezone: val, ..g.get_system() }),
        _ => {}
    }
    g.set_display(d);
    g.set_audio(a);
}

fn toggle(app: &App, key: &str, on: bool) {
    let g = app.global::<SettingsData>();
    let mut d = g.get_display();
    let mut v = g.get_voice();
    let mut a = g.get_audio();
    match key {
        "ambient-night" => d.ambient_night = on,
        "ui-sounds" => a.ui_sounds = on,
        "wake-word" => v.wake_word = on,
        "mic-muted" => v.mic_muted = on,
        "read-reminder" => v.read_reminder = on,
        k if k.starts_with("entity:") => {
            let model = g.get_home().entities;
            let selected = model.iter().filter(|e| e.selected).count();
            for i in 0..model.row_count() {
                let mut e = model.row_data(i).unwrap();
                if format!("entity:{}", e.name) == k && (!on || selected < 4) {
                    e.selected = on;
                    model.set_row_data(i, e);
                }
            }
        }
        _ => {}
    }
    g.set_display(d);
    g.set_audio(a);
    g.set_voice(v);
}

fn start_resolution_countdown(app: &App) {
    app.global::<SettingsData>().set_resolution_confirm(10);
    let weak = app.as_weak();
    RES_TIMER.with(|t| {
        t.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let Some(app) = weak.upgrade() else { return };
            let g = app.global::<SettingsData>();
            let left = g.get_resolution_confirm() - 1;
            g.set_resolution_confirm(left);
            if left <= 0 {
                // Hết giờ: tự huỷ, trả về độ phân giải cũ.
                RES_TIMER.with(|t| t.stop());
                if let Some(prev) = PREV_RES.with(|p| p.borrow_mut().take()) {
                    g.set_display(DisplaySettings { resolution: prev, ..g.get_display() });
                }
            }
        });
    });
}

/// Kịch bản demo: "Đổi độ phân giải".
pub fn change_resolution(app: &App) {
    choose(app, "resolution", "1280x720".into());
}

/// Kịch bản demo: chưa đăng ký voice (mã Enrollment).
pub fn not_enrolled(app: &App) {
    let g = app.global::<SettingsData>();
    g.set_connection(ConnectionSettings {
        enrolled: false,
        enrollment_code: "482 915".into(),
        server_status: "Mất kết nối".into(),
        ..g.get_connection()
    });
}

/// Kịch bản demo: tắt mic.
pub fn mute_mic(app: &App) {
    toggle(app, "mic-muted", true);
}
