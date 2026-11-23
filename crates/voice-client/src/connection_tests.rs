#[path = "../tests/support/fake_server.rs"]
pub(crate) mod fake_server;

use std::time::Duration;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Sink;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
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
    let ota = FakeOtaServer::start(200, serde_json::json!({})).await;
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

fn config_receiver(config: VoiceConfig) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    held_config(config)
}

#[tokio::test(start_paused = true)]
async fn websocket_write_times_out_after_ten_seconds() {
    let mut sink = NeverReadySink;
    let send = connection::send_with_timeout(&mut sink, Message::Text("hello".into()));
    tokio::pin!(send);
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(10)).await;
    assert!(send.await.is_err());
}

struct NeverReadySink;

impl Sink<Message> for NeverReadySink {
    type Error = tokio_tungstenite::tungstenite::Error;

    fn poll_ready(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn start_send(self: Pin<&mut Self>, _item: Message) -> Result<(), Self::Error> {
        unreachable!("send should remain pending")
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _context: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Pending
    }
}

#[tokio::test]
async fn handshake_emits_connecting_then_ready_and_sends_expected_hello() {
    let server = FakeServer::start(ServerAction::SendHello).await;
    let ota = FakeOtaServer::start(
        200,
        serde_json::json!({ "websocket": { "url": server.url, "token": "token-1" } }),
    )
    .await;
    let device = DeviceIdentity::new("device-1", "client-1");
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device,
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
    let ota = FakeOtaServer::start(
        200,
        serde_json::json!({ "websocket": { "url": server.url, "token": "token-1" } }),
    )
    .await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("device-1", "client-1"),
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

fn held_config(config: VoiceConfig) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    let (sender, receiver) = tokio::sync::watch::channel(Some(config));
    tokio::spawn(async move {
        sender.closed().await;
    });
    receiver
}
