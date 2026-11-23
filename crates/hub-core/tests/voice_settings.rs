mod support;
use hub_core::{ConfigStore, UserInput};
use support::Hub;
use voice_client::{ConnectionState, VoiceCommand, VoiceEvent};

#[tokio::test]
async fn saving_and_enrollment_are_reflected_through_the_actor() {
    let mut hub = Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.ui_rx.changed().await.unwrap();
    let identity = ConfigStore::open(hub._dir.path())
        .unwrap()
        .get()
        .voice
        .clone();
    hub.input_tx
        .send(UserInput::SaveVoiceUrl("http://localhost/voice/ota".into()))
        .await
        .unwrap();
    while hub.config_rx.borrow_and_update().is_none() {
        hub.config_rx.changed().await.unwrap();
    }
    assert_eq!(
        hub.config_rx
            .borrow_and_update()
            .as_ref()
            .unwrap()
            .ota_url
            .as_str(),
        "http://localhost/voice/ota"
    );
    let saved = ConfigStore::open(hub._dir.path())
        .unwrap()
        .get()
        .voice
        .clone();
    assert_eq!(saved.device_id, identity.device_id);
    assert_eq!(saved.client_id, identity.client_id);
    assert_eq!(saved.ota_url.as_deref(), Some("http://localhost/voice/ota"));
    hub.input_tx
        .send(UserInput::SaveVoiceUrl("bad url".into()))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice_settings.error.is_none() {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(
        ConfigStore::open(hub._dir.path())
            .unwrap()
            .get()
            .voice
            .ota_url,
        saved.ota_url
    );
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Enrolling))
        .await
        .unwrap();
    hub.voice_tx
        .send(VoiceEvent::Activation { code: "ABC".into() })
        .await
        .unwrap();
    while hub
        .ui_rx
        .borrow_and_update()
        .voice_settings
        .activation
        .as_deref()
        != Some("ABC")
    {
        hub.ui_rx.changed().await.unwrap();
    }
    assert!(hub.ui_rx.borrow().voice_settings.enrolling);
    hub.input_tx.send(UserInput::VoiceConnect).await.unwrap();
    assert!(matches!(
        hub.voice_rx.recv().await,
        Some(VoiceCommand::Connect)
    ));
    hub.voice_tx
        .send(VoiceEvent::Connection(ConnectionState::Ready))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().voice_settings.status_label != "Đã kết nối" {
        hub.ui_rx.changed().await.unwrap();
    }
    assert_eq!(hub.ui_rx.borrow().voice_settings.activation, None);
}
