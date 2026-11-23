use super::{Inputs, to_core_screen, to_ui_screen};
use hub_core::{Focus, UserInput};
use slint::platform::{
    PlatformError, WindowAdapter,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
use slint::{ComponentHandle, Image, ModelRc, VecModel};
use std::rc::Rc;
use std::time::{Duration, Instant};

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
fn every_hub_core_screen_round_trips_through_slint() {
    let screens = [
        hub_core::Screen::Center,
        hub_core::Screen::Menu,
        hub_core::Screen::Ambient,
        hub_core::Screen::Weather,
        hub_core::Screen::Calendar,
        hub_core::Screen::Home,
        hub_core::Screen::Timers,
        hub_core::Screen::Countdowns,
        hub_core::Screen::Reminders,
        hub_core::Screen::Photos,
        hub_core::Screen::Settings,
    ];

    for core_screen in screens {
        let slint_screen = to_ui_screen(core_screen);
        assert_eq!(to_core_screen(slint_screen), Some(core_screen));
    }
    assert_eq!(to_core_screen(crate::Screen::FirstBoot), None);
}

#[test]
fn activity_is_sent_once_during_a_one_second_burst() {
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    let mut inputs = Inputs::new(tx);
    let start = Instant::now();

    for tenth in 0..10 {
        inputs.activity_at(start + Duration::from_millis(tenth * 100));
    }

    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
    assert!(rx.try_recv().is_err());

    inputs.activity_at(start + Duration::from_secs(1));
    assert!(matches!(rx.try_recv(), Ok(UserInput::Activity)));
}

#[test]
fn home_and_photo_callbacks_map_to_hub_inputs() {
    let app = app();
    let (tx, mut rx) = tokio::sync::mpsc::channel(16);
    super::connect_inputs(&app, tx, crate::ui::bridge::BridgeState::default());

    let photos = app.global::<crate::PhotosData>();
    let photo = |key: &str, pinned| crate::Photo {
        key: key.into(),
        thumb: Image::default(),
        full: Image::default(),
        caption: String::new().into(),
        source: String::new().into(),
        time: String::new().into(),
        pinned,
        state: crate::PhotoState::Ready,
    };
    let model = ModelRc::from(Rc::new(VecModel::from(vec![
        photo("p1", false),
        photo("p2", true),
    ])));
    photos.set_photos(model.clone());
    photos.set_viewer(model);
    photos.set_viewer_index(0);

    app.global::<crate::HomeData>()
        .invoke_toggle("light.living".into());
    photos.invoke_open("p1".into());
    photos.invoke_next();
    photos.invoke_previous();
    photos.invoke_close_viewer();
    photos.invoke_toggle_pin("p1".into());
    photos.invoke_remove("p2".into());

    fn next_action(rx: &mut tokio::sync::mpsc::Receiver<UserInput>) -> UserInput {
        loop {
            let input = rx.try_recv().expect("expected bridge input");
            if !matches!(&input, UserInput::Activity) {
                return input;
            }
        }
    }

    assert!(matches!(next_action(&mut rx), UserInput::ToggleHome(key) if key == "light.living"));
    assert!(
        matches!(next_action(&mut rx), UserInput::Focus(Some(Focus::Photo(key))) if key == "p1")
    );
    assert!(
        matches!(next_action(&mut rx), UserInput::Focus(Some(Focus::Photo(key))) if key == "p2")
    );
    assert!(
        matches!(next_action(&mut rx), UserInput::Focus(Some(Focus::Photo(key))) if key == "p2")
    );
    assert!(matches!(next_action(&mut rx), UserInput::Focus(None)));
    assert!(matches!(next_action(&mut rx), UserInput::PinPhoto { id, pinned: true } if id == "p1"));
    assert!(matches!(next_action(&mut rx), UserInput::RemovePhoto { id } if id == "p2"));
    assert!(rx.try_recv().is_err());
}
