//! Bảng kịch bản của Demo panel. Ticket sau đăng ký kịch bản bằng một dòng.
use crate::{AmbientData, App, Nav, Screen, StatusBar, Theme, Voice, VoiceState};
use slint::ComponentHandle;

pub type Scenario = (&'static str, fn(&App));

/// Mở một màn từ kịch bản (tắt bảng mẫu nếu đang mở).
pub(super) fn show(app: &App, screen: Screen) {
    app.set_show_sample(false);
    app.global::<Nav>().set_screen(screen);
}

pub fn all() -> Vec<Scenario> {
    vec![
        ("Mở menu", |app| {
            show(app, Screen::Menu)
        }),
        ("Về trung tâm", |app| {
            show(app, Screen::Center)
        }),
        ("Mở Thời tiết (tool)", |app| {
            show(app, Screen::Weather)
        }),
        ("Tự về trung tâm", |app| app.global::<Nav>().set_screen(Screen::Center)),
        ("Menu: không có gì chạy", super::fixtures::menu::idle),
        ("Menu: đủ 8 dòng", super::fixtures::init),
        ("Offline bật/tắt", |app| {
            let bar = app.global::<StatusBar>();
            bar.set_offline(!bar.get_offline());
        }),
        ("Nhà: bật đèn phòng khách", |app| {
            show(app, Screen::Home);
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
        ("Hết giờ trứng", |app| super::fixtures::alerts::alert(app, crate::AlertKind::Timer, "Trứng", "", "")),
        ("Reminder uống thuốc", |app| super::fixtures::alerts::alert(app, crate::AlertKind::Reminder, "Nhắc", "08:00", "Uống thuốc huyết áp")),
        ("Hai báo liên tiếp", |app| {
            super::fixtures::alerts::alert(app, crate::AlertKind::Timer, "Trứng", "", "");
            super::fixtures::alerts::alert(app, crate::AlertKind::Reminder, "Nhắc", "08:00", "Uống thuốc huyết áp")
        }),
        ("Báo khi đang Ambient", |app| {
            app.global::<Nav>().set_screen(Screen::Ambient);
            super::fixtures::alerts::alert(app, crate::AlertKind::Timer, "Trứng", "", "")
        }),
        ("Độ sáng 30%", |app| super::fixtures::alerts::slider(app, crate::SliderKind::Brightness, 0.3)),
        ("Âm lượng 80%", |app| super::fixtures::alerts::slider(app, crate::SliderKind::Volume, 0.8)),
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
        ("Lần đầu: 3 bước", |app| super::fixtures::first_boot::run(app, false)),
        ("Lần đầu: sai mật khẩu", |app| super::fixtures::first_boot::run(app, true)),
        ("Chữ nhỏ", |app| app.global::<Theme>().set_text_scale(0.85)),
        ("Chữ vừa", |app| app.global::<Theme>().set_text_scale(1.0)),
        ("Mở Setting", |app| {
            show(app, Screen::Settings)
        }),
        ("Đổi độ phân giải", super::fixtures::settings::change_resolution),
        ("Chưa đăng ký voice", super::fixtures::settings::not_enrolled),
        ("Setting: tắt mic", super::fixtures::settings::mute_mic),
        ("Setting: xoá toàn bộ ảnh (mở Dữ liệu)", |app| show(app, Screen::Settings)),
        ("Người dùng nói", super::fixtures::response::user_says),
        ("Agent trả lời", super::fixtures::response::agent_reply),
        ("Tool server: lịch", super::fixtures::response::tool_server),
        ("Tool lỗi", super::fixtures::response::tool_failed),
        ("Im lặng → mờ", super::fixtures::response::silence),
        ("Sinh vào ô 3", super::fixtures::spawn::spawn_cell3),
        ("Lưu → thành công", super::fixtures::spawn::save_ok),
        ("Lưu → lỗi", super::fixtures::spawn::save_fail),
        ("Thẻ nổi", super::fixtures::spawn::floating),
        ("Thẻ nổi mờ", super::fixtures::spawn::floating_fade),
        ("Ảnh: agent gửi 3 ảnh Đà Lạt", super::fixtures::photos::agent_sends),
        ("Ảnh: lỗi", super::fixtures::photos::failed),
        ("Ảnh: tiếp", super::fixtures::photos::next),
        ("Ảnh: trước", super::fixtures::photos::previous),
        ("Ảnh: giữ ảnh này", super::fixtures::photos::pin_current),
        ("Ảnh: xem lại lúc nãy", super::fixtures::photos::reopen),
        ("Lịch: Hôm nay có gì?", super::fixtures::calendar::scenarios::today),
        ("Lịch: Thứ Sáu có gì?", super::fixtures::calendar::scenarios::friday),
        ("Lịch: Lịch tiếp theo", super::fixtures::calendar::scenarios::next),
        ("Lịch: Tháng sau có gì?", super::fixtures::calendar::scenarios::next_month),
        ("Lịch: Tạo khám răng → ok", super::fixtures::calendar::scenarios::create_ok),
        ("Lịch: Tạo → lỗi", super::fixtures::calendar::scenarios::create_fail),
        ("Lịch: Offline", super::fixtures::calendar::scenarios::offline),
        ("Thời tiết: quang ngày", |app| super::fixtures::weather::background(app, crate::WeatherBg::Clear, true)),
        ("Thời tiết: quang đêm", |app| super::fixtures::weather::background(app, crate::WeatherBg::Clear, false)),
        ("Thời tiết: nhiều mây", |app| super::fixtures::weather::background(app, crate::WeatherBg::Cloudy, true)),
        ("Thời tiết: mưa", |app| super::fixtures::weather::background(app, crate::WeatherBg::Rain, true)),
        ("Thời tiết: giông", |app| super::fixtures::weather::background(app, crate::WeatherBg::Storm, false)),
        ("Thời tiết: sương mù", |app| super::fixtures::weather::background(app, crate::WeatherBg::Fog, true)),
        ("Mai có mưa không?", super::fixtures::weather::tomorrow_rain),
        ("Chiều nay mấy độ?", super::fixtures::weather::afternoon),
        ("Không khí thế nào?", super::fixtures::weather::air),
        ("Thời tiết Đà Nẵng", super::fixtures::weather::danang),
        ("Thời tiết offline", super::fixtures::weather::offline),
        ("Đếm ngược đến Tết", super::fixtures::countdowns::add_tet),
        ("Sinh nhật 12/3", super::fixtures::countdowns::add_birthday),
        ("Qua nửa đêm", super::fixtures::countdowns::midnight),
        ("Còn < 24 giờ", super::fixtures::countdowns::live),
        ("Hôm nay là Tết", super::fixtures::countdowns::today),
        ("Chuyến đi đã qua", super::fixtures::countdowns::passed),
        ("Còn bao nhiêu ngày đến Tết?", super::fixtures::countdowns::focus_tet),
        ("Xoá chuyến đi", super::fixtures::countdowns::remove_trip),
        ("Trung tâm: thêm hẹn giờ", super::fixtures::center::add_timer),
        ("Trung tâm: hết hẹn giờ", super::fixtures::center::end_timer),
        ("Trung tâm: hôm nay là Tết", super::fixtures::center::countdown_today),
        ("Trung tâm: hỏi mai có mưa", super::fixtures::center::focus_weather),
        ("Trung tâm: offline", super::fixtures::center::stale),
        ("Reminder: nhắc uống thuốc 21:00", |app| { app.global::<Nav>().set_screen(Screen::Reminders); super::fixtures::reminders::add_medicine(app) }),
        ("Reminder: mỗi sáng 7 giờ", |app| { app.global::<Nav>().set_screen(Screen::Reminders); super::fixtures::reminders::add_gym(app) }),
        ("Reminder: tối nay có gì?", super::fixtures::reminders::focus_tonight),
        ("Reminder: huỷ uống thuốc", super::fixtures::reminders::cancel_medicine),
        ("Reminder: hoãn 10 phút", super::fixtures::reminders::postpone),
        ("Reminder: một mục đã lỡ", super::fixtures::reminders::missed),
        ("Reminder: trống", super::fixtures::reminders::empty),
        ("Hẹn giờ: luộc trứng 7 phút", super::fixtures::timers::boil_egg),
        ("Hẹn giờ: thêm một cái", super::fixtures::timers::add_another),
        ("Hẹn giờ: 10 giây cuối", super::fixtures::timers::final_seconds),
        ("Hẹn giờ: tạm dừng / tiếp tục", super::fixtures::timers::pause_resume),
        ("Hẹn giờ: thêm 2 phút", super::fixtures::timers::add_two_minutes),
        ("Hẹn giờ: huỷ trứng", super::fixtures::timers::cancel_egg),
        ("Hẹn giờ: hết giờ", super::fixtures::timers::finish),
        ("Hẹn giờ: trứng còn bao lâu?", super::fixtures::timers::focus_egg),
        ("Chữ lớn", |app| app.global::<Theme>().set_text_scale(1.25)),
    ]
}
