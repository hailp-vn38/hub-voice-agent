#[path = "../tests/support/fake_server.rs"]
mod fake_server;

use std::time::Duration;

use tokio::time::timeout;
use url::Url;

use crate::{DeviceIdentity, VoiceConfig, VoiceEvent, connection};
use fake_server::{FakeOtaServer, FakeServer, ServerAction};

#[tokio::test]
async fn public_spawn_discovers_websocket_url_and_token_from_ota() {
    let websocket = FakeServer::start(ServerAction::SendHello).await;
    let ota = FakeOtaServer::start(
        200,
        serde_json::json!({
            "websocket": { "url": websocket.url, "token": "ota-token" }
        }),
    )
    .await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("ota-device", "ota-client"),
        tools: Vec::new(),
    };
    let (_commands, mut events) = crate::spawn(config_receiver(config));

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));

    let request = timeout(Duration::from_secs(2), ota.received)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(request.method, "GET");
    assert_eq!(request.device_id, "ota-device");
    assert_eq!(request.client_id, "ota-client");
    assert!(request.body.is_empty());
    let received = timeout(Duration::from_secs(2), websocket.received)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(received.authorization, "Bearer ota-token");
    assert_eq!(received.device_id, "ota-device");
    assert_eq!(received.client_id, "ota-client");
}

#[tokio::test]
async fn ota_http_error_emits_offline_without_opening_websocket() {
    let ota = FakeOtaServer::start(403, serde_json::json!({})).await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("blocked-device", "ota-client"),
        tools: Vec::new(),
    };
    let (_commands, mut events) = crate::spawn(config_receiver(config));

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));
}

#[tokio::test]
async fn ota_response_without_websocket_emits_offline() {
    let ota =
        FakeOtaServer::start(200, serde_json::json!({ "activation": { "code": "123" } })).await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("pending-device", "ota-client"),
        tools: Vec::new(),
    };
    let (_commands, mut events) = crate::spawn(config_receiver(config));

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));
}

fn config_receiver(config: VoiceConfig) -> tokio::sync::watch::Receiver<VoiceConfig> {
    tokio::sync::watch::channel(config).1
}

#[tokio::test]
async fn handshake_emits_connecting_then_ready_and_sends_expected_hello() {
    let server = FakeServer::start(ServerAction::SendHello).await;
    let device = DeviceIdentity::new("device-1", "client-1");
    let (_commands, mut events) = connection::spawn_with_websocket(
        Url::parse(&server.url).unwrap(),
        "token-1".into(),
        device,
    );

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));

    let received = timeout(Duration::from_secs(2), server.received)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(received.authorization, "Bearer token-1");
    assert_eq!(received.device_id, "device-1");
    assert_eq!(received.client_id, "client-1");
    assert_eq!(received.hello["type"], "hello");
    assert_eq!(received.hello["version"], 1);
    assert_eq!(received.hello["transport"], "websocket");
    assert_eq!(
        received.hello["audio_params"],
        serde_json::json!({
            "format": "opus", "sample_rate": 16000, "channels": 1, "frame_duration": 60
        })
    );
    assert_eq!(
        received.hello["features"],
        serde_json::json!({
            "mcp": true, "tool_status": true, "direct_tts": true, "aec": true
        })
    );
}

#[tokio::test]
async fn server_protocol_close_after_client_hello_emits_offline() {
    let server = FakeServer::start(ServerAction::CloseProtocolError).await;
    let (_commands, mut events) = connection::spawn_with_websocket(
        Url::parse(&server.url).unwrap(),
        "token-1".into(),
        DeviceIdentity::new("device-1", "client-1"),
    );

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));
}
