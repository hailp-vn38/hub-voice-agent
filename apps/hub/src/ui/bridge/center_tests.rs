use hub_core::{
    Screen as CoreScreen, UserInput,
    present::{
        AmbientView, CenterCard as CoreCenterCard, CenterCountdown as CoreCenterCountdown,
        CenterEvent as CoreCenterEvent, CenterReminder as CoreCenterReminder,
        CenterTimer as CoreCenterTimer, CenterView, CenterWeather as CoreCenterWeather,
        HomeKind as CoreHomeKind, HomeTileView, MenuView,
    },
};
use slint::platform::{
    PlatformError, WindowAdapter,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use slint::{ComponentHandle, Model};
use std::rc::Rc;

thread_local! {
    static TEST_WINDOW: Rc<MinimalSoftwareWindow> =
        MinimalSoftwareWindow::new(RepaintBufferType::ReusedBuffer);
}

struct TestPlatform;

impl slint::platform::Platform for TestPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(TEST_WINDOW.with(|window| window.clone()))
    }
}

fn app() -> crate::App {
    slint::platform::set_platform(Box::new(TestPlatform)).expect("install test platform");
    crate::App::new().expect("create Slint app")
}

#[test]
fn center_menu_and_ambient_views_populate_their_contracts() {
    let app = app();
    let center = CenterView {
        time: "14:32".into(),
        date: "Thứ hai, 23/11".into(),
        weather: CoreCenterWeather {
            icon: "rain".into(),
            temp: "24°".into(),
            summary: "Mưa nhẹ".into(),
            updated: "14:30".into(),
            stale: true,
        },
        today: vec![CoreCenterEvent {
            key: "event-1".into(),
            time: "15:00".into(),
            title: "Họp".into(),
            now: false,
        }],
        reminders: vec![CoreCenterReminder {
            key: "reminder-1".into(),
            time: "16:00".into(),
            text: "Uống thuốc".into(),
        }],
        timer: CoreCenterTimer {
            visible: true,
            key: "42".into(),
            label: "Trứng".into(),
            remaining: "0:59".into(),
            progress: 0.25,
        },
        countdown: CoreCenterCountdown {
            visible: true,
            key: "7".into(),
            label: "Sinh nhật".into(),
            value: "3 ngày".into(),
            today: false,
        },
        home: vec![HomeTileView {
            key: "light.kitchen".into(),
            name: "Đèn bếp".into(),
            kind: CoreHomeKind::Light,
            on: true,
            value: String::new(),
            pending: false,
            stale: false,
        }],
        home_safe: false,
        focus: CoreCenterCard::Calendar,
        focus_serial: 12,
    };
    let menu = MenuView {
        weather: "24° · Mưa nhẹ".into(),
        calendar: "1 sự kiện hôm nay".into(),
        home: "4 thiết bị".into(),
        timers: "1 hẹn giờ đang chạy".into(),
        countdowns: "2 mốc sắp tới".into(),
        reminders: "3 lời nhắc sắp tới".into(),
        photos: "5 ảnh".into(),
        settings: "Đã kết nối".into(),
    };
    let ambient = AmbientView {
        time: "14:32".into(),
        temp: "24°".into(),
        icon: "rain".into(),
        night: false,
    };

    super::center::apply_center(&app, &center);
    super::center::apply_menu(&app, &menu);
    super::center::apply_ambient(&app, &ambient);

    let center_ui = app.global::<crate::Center>();
    assert_eq!(center_ui.get_time(), "14:32");
    assert_eq!(center_ui.get_date(), "Thứ hai, 23/11");
    assert_eq!(center_ui.get_weather().temp, "24°");
    assert_eq!(center_ui.get_weather().stale, true);
    assert_eq!(center_ui.get_today().row_count(), 1);
    assert_eq!(center_ui.get_today().row_data(0).unwrap().title, "Họp");
    assert_eq!(
        center_ui.get_reminders().row_data(0).unwrap().text,
        "Uống thuốc"
    );
    assert_eq!(center_ui.get_timer().key, "42");
    assert_eq!(center_ui.get_countdown().label, "Sinh nhật");
    assert_eq!(center_ui.get_home().row_data(0).unwrap().name, "Đèn bếp");
    assert!(!center_ui.get_home_safe());
    assert_eq!(center_ui.get_focus(), crate::CenterCard::Calendar);
    assert_eq!(center_ui.get_focus_serial(), 12);

    let menu_ui = app.global::<crate::Menu>();
    assert_eq!(menu_ui.get_weather(), "24° · Mưa nhẹ");
    assert_eq!(menu_ui.get_calendar(), "1 sự kiện hôm nay");
    assert_eq!(menu_ui.get_home(), "4 thiết bị");
    assert_eq!(menu_ui.get_timers(), "1 hẹn giờ đang chạy");
    assert_eq!(menu_ui.get_countdowns(), "2 mốc sắp tới");
    assert_eq!(menu_ui.get_reminders(), "3 lời nhắc sắp tới");
    assert_eq!(menu_ui.get_photos(), "5 ảnh");
    assert_eq!(menu_ui.get_settings(), "Đã kết nối");

    let ambient_ui = app.global::<crate::AmbientData>();
    assert_eq!(ambient_ui.get_time(), "14:32");
    assert_eq!(ambient_ui.get_temp(), "24°");
    assert_eq!(ambient_ui.get_icon(), "rain");
    assert!(!ambient_ui.get_night());
}

#[test]
fn center_and_ambient_callbacks_send_core_inputs_and_activity() {
    let app = app();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    super::connect_inputs(&app, tx, crate::ui::bridge::BridgeState::default());

    app.global::<crate::Center>()
        .invoke_open(crate::Screen::Weather);
    app.global::<crate::Center>()
        .invoke_toggle_home("light.kitchen".into());
    app.global::<crate::AmbientData>().invoke_wake();

    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
    assert!(matches!(
        rx.try_recv(),
        Ok(UserInput::Open(CoreScreen::Weather))
    ));
    assert!(matches!(
        rx.try_recv(),
        Ok(UserInput::ToggleHome(key)) if key == "light.kitchen"
    ));
    assert!(rx.try_recv().is_err());
}
