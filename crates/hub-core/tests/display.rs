mod support;
use hub_core::{Device, SettingChange, UserInput};
use std::sync::{Arc, Mutex};
use support::Hub;

#[derive(Default)]
struct FakeDevice {
    mode: Mutex<String>,
    brightness: Mutex<Vec<f32>>,
}

impl Device for FakeDevice {
    fn set_brightness(&self, level: f32) {
        self.brightness.lock().unwrap().push(level);
    }
    fn resolutions(&self) -> Vec<String> {
        vec!["1280x720".into(), "1920x1080".into()]
    }
    fn current_resolution(&self) -> Option<String> {
        Some(self.mode.lock().unwrap().clone())
    }
    fn set_resolution(&self, mode: &str) -> anyhow::Result<()> {
        *self.mode.lock().unwrap() = mode.into();
        Ok(())
    }
}

fn start(device: Arc<FakeDevice>) -> Hub {
    *device.mode.lock().unwrap() = "1280x720".into();
    Hub::start_with_device(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
        device,
    )
}

#[tokio::test(start_paused = true)]
async fn resolution_is_reverted_unless_kept() {
    let device = Arc::new(FakeDevice::default());
    let mut hub = start(device.clone());
    hub.ui_rx.changed().await.unwrap();
    assert_eq!(hub.ui_rx.borrow().display.resolutions.len(), 2);

    hub.input_tx
        .send(UserInput::Setting(SettingChange::Resolution(
            "1920x1080".into(),
        )))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().display.resolution_confirm == 0 {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(*device.mode.lock().unwrap(), "1920x1080");

    while hub.ui_rx.borrow_and_update().display.resolution_confirm != 0 {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(*device.mode.lock().unwrap(), "1280x720");
    assert_eq!(
        hub.ui_rx.borrow().display.resolution.as_deref(),
        Some("1280x720")
    );
    assert_eq!(hub.ui_rx.borrow().settings.resolution, None);
}

#[tokio::test(start_paused = true)]
async fn kept_resolution_is_saved() {
    let device = Arc::new(FakeDevice::default());
    let mut hub = start(device.clone());
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Resolution(
            "1920x1080".into(),
        )))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().display.resolution_confirm == 0 {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.input_tx.send(UserInput::KeepResolution).await.unwrap();
    while hub.ui_rx.borrow_and_update().display.resolution_confirm != 0 {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(*device.mode.lock().unwrap(), "1920x1080");
    let saved = hub_core::ConfigStore::open(hub._dir.path()).unwrap();
    assert_eq!(saved.get().device.resolution.as_deref(), Some("1920x1080"));
}

#[tokio::test(start_paused = true)]
async fn brightness_follows_settings_and_night() {
    let device = Arc::new(FakeDevice::default());
    let mut hub = start(device.clone());
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Brightness(0.5)))
        .await
        .unwrap();
    while device.brightness.lock().unwrap().last() != Some(&0.5) {
        hub.ui_rx.changed().await.unwrap();
    }
}
