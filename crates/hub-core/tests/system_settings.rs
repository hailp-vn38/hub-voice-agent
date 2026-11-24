mod support;
use hub_core::{ConfigStore, Device, PowerAction, SettingChange, UserInput};
use std::sync::{Arc, Mutex};
use support::Hub;

#[derive(Default)]
struct FakeDevice {
    power: Mutex<Vec<PowerAction>>,
}

impl Device for FakeDevice {
    fn set_brightness(&self, _: f32) {}
    fn resolutions(&self) -> Vec<String> {
        Vec::new()
    }
    fn current_resolution(&self) -> Option<String> {
        None
    }
    fn set_resolution(&self, _: &str) -> anyhow::Result<()> {
        Ok(())
    }
    fn power(&self, action: PowerAction) -> anyhow::Result<()> {
        self.power.lock().unwrap().push(action);
        Ok(())
    }
}

fn start(device: Arc<FakeDevice>) -> Hub {
    Hub::start_with_device(
        "2026-11-24T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
        device,
    )
}

#[tokio::test(start_paused = true)]
async fn power_actions_reach_the_device() {
    let device = Arc::new(FakeDevice::default());
    let mut hub = start(device.clone());
    hub.ui_rx.changed().await.unwrap();
    for action in [
        PowerAction::RestartApp,
        PowerAction::Reboot,
        PowerAction::Shutdown,
    ] {
        hub.input_tx.send(UserInput::Power(action)).await.unwrap();
    }
    while device.power.lock().unwrap().len() < 3 {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(
        *device.power.lock().unwrap(),
        [
            PowerAction::RestartApp,
            PowerAction::Reboot,
            PowerAction::Shutdown
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn timezone_changes_the_displayed_time_and_is_saved() {
    let mut hub = start(Arc::new(FakeDevice::default()));
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Timezone("UTC".into())))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().now.hour() != 5 {
        hub.ui_rx.changed().await.unwrap();
    }
    let saved = ConfigStore::open(hub._dir.path()).unwrap();
    assert_eq!(saved.get().device.timezone.as_deref(), Some("UTC"));
}

#[tokio::test(start_paused = true)]
async fn unknown_timezone_is_rejected_and_not_saved() {
    let mut hub = start(Arc::new(FakeDevice::default()));
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::Timezone(
            "Not/AZone".into(),
        )))
        .await
        .unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::UiSounds(false)))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().settings.ui_sounds {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(hub.ui_rx.borrow().settings.timezone, None);
    assert_eq!(hub.ui_rx.borrow().now.hour(), 12);
}
