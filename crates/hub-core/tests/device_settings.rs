mod support;
use hub_core::{ConfigStore, SettingChange, TextSize, UserInput};
use support::Hub;

#[tokio::test]
async fn setting_changes_reach_ui_state_and_config() {
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Setting(SettingChange::TextSize(TextSize::Large)))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().settings.text_size != TextSize::Large {
        hub.ui_rx.changed().await.unwrap();
    }
    let saved = ConfigStore::open(hub._dir.path()).unwrap();
    assert_eq!(saved.get().device.text_size, TextSize::Large);
}
