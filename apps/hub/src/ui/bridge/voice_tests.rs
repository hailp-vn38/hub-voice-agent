use crate::audio::Levels;
use slint::ComponentHandle;
use slint::platform::{
    PlatformError, WindowAdapter,
    software_renderer::{MinimalSoftwareWindow, RepaintBufferType},
};
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
fn audio_levels_update_the_voice_levels_contract() {
    let app = app();

    super::apply_levels(
        &app,
        Levels {
            mic: 0.25,
            tts: 0.75,
        },
    );

    let levels = app.global::<crate::VoiceLevels>();
    assert_eq!(levels.get_mic(), 0.25);
    assert_eq!(levels.get_tts(), 0.75);
}
