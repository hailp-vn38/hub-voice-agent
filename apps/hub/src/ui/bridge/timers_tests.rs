use hub_core::{
    TimerInput, UserInput,
    present::{AlertKind as CoreAlertKind, AlertView, TimerCardView, TimerPhase, TimersView},
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
fn timers_view_populates_contract_and_focus() {
    let app = app();
    let view = TimersView {
        items: vec![TimerCardView {
            key: "42".to_string(),
            label: "Trứng".to_string(),
            remaining: "0:59".to_string(),
            detail: "1 phút · xong lúc 14:32".to_string(),
            progress: 0.5,
            phase: TimerPhase::FinalSeconds,
        }],
        title: "Hẹn giờ · 1 đang chạy".to_string(),
        focus: Some("42".to_string()),
        serial: 7,
    };

    super::timers::apply_timers(&app, &view);

    let timers = app.global::<crate::TimersData>();
    assert_eq!(timers.get_title(), "Hẹn giờ · 1 đang chạy");
    assert_eq!(timers.get_focus(), "42");
    assert_eq!(timers.get_focus_serial(), 7);
    let cards = timers.get_timers();
    assert_eq!(cards.row_count(), 1);
    let card = cards.row_data(0).expect("timer card");
    assert_eq!(card.key, "42");
    assert_eq!(card.label, "Trứng");
    assert_eq!(card.remaining, "0:59");
    assert_eq!(card.progress, 0.5);
    assert_eq!(card.phase, crate::TimerPhase::FinalSeconds);
}

#[test]
fn alert_view_maps_and_none_hides_without_clearing_fade_data() {
    let app = app();
    let alert = AlertView {
        kind: CoreAlertKind::Timer,
        title: "Hẹn giờ đã xong".to_string(),
        time: "14:32".to_string(),
        body: "Trứng".to_string(),
        serial: 9,
    };

    super::timers::apply_alert(&app, Some(&alert));
    let data = app.global::<crate::AlertData>();
    assert_eq!(data.get_kind(), crate::AlertKind::Timer);
    assert_eq!(data.get_serial(), 9);
    assert_eq!(data.get_title(), "Hẹn giờ đã xong");
    assert_eq!(data.get_time(), "14:32");
    assert_eq!(data.get_body(), "Trứng");

    super::timers::apply_alert(&app, None);
    assert_eq!(data.get_kind(), crate::AlertKind::None);
    assert_eq!(data.get_serial(), 9);
    assert_eq!(data.get_body(), "Trứng");
}

#[test]
fn timer_and_alert_callbacks_map_to_core_inputs_and_ignore_invalid_ids() {
    let app = app();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    super::connect_inputs(&app, tx, super::BridgeState::default());

    let timers = app.global::<crate::TimersData>();
    timers.invoke_quick_start(1);
    timers.invoke_toggle_pause("42".into());
    timers.invoke_add_minute("bad-id".into());
    timers.invoke_cancel("123".into());
    app.global::<crate::AlertData>().invoke_dismiss();

    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
    assert!(matches!(
        rx.try_recv(),
        Ok(UserInput::Timer(TimerInput::QuickStart(1)))
    ));
    assert!(matches!(
        rx.try_recv(),
        Ok(UserInput::Timer(TimerInput::TogglePause(42)))
    ));
    assert!(matches!(
        rx.try_recv(),
        Ok(UserInput::Timer(TimerInput::Cancel(123)))
    ));
    assert!(matches!(rx.try_recv(), Ok(UserInput::DismissAlert)));
    assert!(rx.try_recv().is_err());
}
