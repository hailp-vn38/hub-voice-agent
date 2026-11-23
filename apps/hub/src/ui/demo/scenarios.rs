//! Bảng kịch bản của Demo panel. Ticket sau đăng ký kịch bản bằng một dòng.
use crate::{AmbientData, App, Nav, Screen, StatusBar, Theme, Voice, VoiceState};
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
        ("Menu: không có gì chạy", super::fixtures::menu_idle),
        ("Menu: đủ 8 dòng", super::fixtures::init),
        ("Offline bật/tắt", |app| {
            let bar = app.global::<StatusBar>();
            bar.set_offline(!bar.get_offline());
        }),
        ("Nhà: bật đèn phòng khách", |app| {
            app.set_show_sample(false);
            app.global::<Nav>().set_screen(Screen::Home);
            super::fixtures::home::turn_on_living(app)
        }),
        ("Nhà: mất kết nối HA", |app| super::fixtures::home::disconnect(app)),
        ("Nhà: chưa cấu hình", |app| super::fixtures::home::unconfigure(app)),
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
        ("Lần đầu: 3 bước", |app| super::fixtures::run_first_boot(app, false)),
        ("Lần đầu: sai mật khẩu", |app| super::fixtures::run_first_boot(app, true)),
        ("Chữ nhỏ", |app| app.global::<Theme>().set_text_scale(0.85)),
        ("Chữ vừa", |app| app.global::<Theme>().set_text_scale(1.0)),
        ("Mở Setting", |app| {
            app.set_show_sample(false);
            app.global::<Nav>().set_screen(Screen::Settings)
        }),
        ("Đổi độ phân giải", super::settings::change_resolution),
        ("Chưa đăng ký voice", super::settings::not_enrolled),
        ("Tắt mic", super::settings::mute_mic),
        ("Sinh vào ô 3", super::spawn::spawn_cell3),
        ("Lưu → thành công", super::spawn::save_ok),
        ("Lưu → lỗi", super::spawn::save_fail),
        ("Thẻ nổi", super::spawn::floating),
        ("Thẻ nổi mờ", super::spawn::floating_fade),
        ("Hẹn giờ: luộc trứng 7 phút", super::timers::boil_egg),
        ("Hẹn giờ: thêm một cái", super::timers::add_another),
        ("Hẹn giờ: 10 giây cuối", super::timers::final_seconds),
        ("Hẹn giờ: tạm dừng / tiếp tục", super::timers::pause_resume),
        ("Hẹn giờ: thêm 2 phút", super::timers::add_two_minutes),
        ("Hẹn giờ: huỷ trứng", super::timers::cancel_egg),
        ("Hẹn giờ: hết giờ", super::timers::finish),
        ("Hẹn giờ: trứng còn bao lâu?", super::timers::focus_egg),
        ("Chữ lớn", |app| app.global::<Theme>().set_text_scale(1.25)),
    ]
}
