#[path = "../tests/support/fake_server.rs"]
mod fake_server;

use std::time::Duration;

use tokio::time::timeout;
use url::Url;

use crate::{DeviceIdentity, VoiceEvent, connection};
use fake_server::{FakeServer, ServerAction};

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
