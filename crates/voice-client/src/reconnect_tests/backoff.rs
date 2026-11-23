use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use url::Url;

use super::support::failing_ota;
use crate::connection::tests::fake_server;
use crate::{DeviceIdentity, VoiceConfig, VoiceEvent, spawn_with_jitter};

#[tokio::test(start_paused = true)]
async fn ota_failure_emits_offline_then_retries_after_injected_backoff() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (requests_tx, mut requests_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut request = vec![0; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let _ = requests_tx.send(());
            stream
                .write_all(
                    b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .unwrap();
        }
    });

    let config = VoiceConfig {
        ota_url: Url::parse(&format!("http://{address}/voice/ota/")).unwrap(),
        device: DeviceIdentity::new("retry-device", "retry-client"),
        tools: Vec::new(),
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = spawn_with_jitter(config_rx, |nominal| {
        Duration::from_millis(nominal.as_millis() as u64 * 120 / 100)
    });

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(requests_rx.recv().await.is_some());
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::Connection(crate::ConnectionState::Offline { retry_in })) if retry_in == Duration::from_millis(1200))
    );

    tokio::time::advance(Duration::from_millis(1200)).await;
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(requests_rx.recv().await.is_some());
}

#[tokio::test(start_paused = true)]
async fn ota_failures_double_backoff_to_sixty_seconds() {
    let (ota_url, mut requests) = failing_ota().await;
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("backoff-device", "backoff-client"),
        tools: Vec::new(),
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = spawn_with_jitter(config_rx, |nominal| nominal);

    for delay in [1, 2, 4, 8, 16, 32, 60, 60] {
        assert!(matches!(
            events.recv().await,
            Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
        ));
        assert!(requests.recv().await.is_some());
        assert!(
            matches!(events.recv().await, Some(VoiceEvent::Connection(crate::ConnectionState::Offline { retry_in })) if retry_in == Duration::from_secs(delay))
        );
        tokio::time::advance(Duration::from_secs(delay)).await;
    }
}

#[tokio::test(start_paused = true)]
async fn disconnect_after_ready_resets_backoff_to_one_second() {
    let websocket = fake_server::FakeServer::start(fake_server::ServerAction::SendHello).await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let websocket_url = websocket.url.clone();
    let (requests_tx, mut requests_rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut request = vec![0; 2048];
            let _ = stream.read(&mut request).await.unwrap();
            let _ = requests_tx.send(());
            let body =
                serde_json::json!({ "websocket": { "url": websocket_url, "token": "token" } })
                    .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        }
    });
    let config = VoiceConfig {
        ota_url: Url::parse(&format!("http://{address}/voice/ota/")).unwrap(),
        device: DeviceIdentity::new("reset-device", "reset-client"),
        tools: Vec::new(),
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = spawn_with_jitter(config_rx, |nominal| nominal);

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(requests_rx.recv().await.is_some());
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::Connection(crate::ConnectionState::Offline { retry_in })) if retry_in == Duration::from_secs(1))
    );
    tokio::time::advance(Duration::from_secs(1)).await;
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(requests_rx.recv().await.is_some());
}

#[tokio::test(start_paused = true)]
async fn jitter_never_pushes_retry_past_sixty_seconds() {
    let (ota_url, mut requests) = failing_ota().await;
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("cap-device", "cap-client"),
        tools: Vec::new(),
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = spawn_with_jitter(config_rx, |nominal| nominal * 120 / 100);

    let mut largest = Duration::ZERO;
    for _ in 0..9 {
        assert!(matches!(
            events.recv().await,
            Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
        ));
        assert!(requests.recv().await.is_some());
        let Some(VoiceEvent::Connection(crate::ConnectionState::Offline { retry_in })) =
            events.recv().await
        else {
            panic!("expected offline")
        };
        largest = largest.max(retry_in);
        tokio::time::advance(retry_in).await;
    }
    assert_eq!(largest, Duration::from_secs(60));
}

/// Real clock: the stability threshold is measured on socket IO.
#[tokio::test]
async fn backoff_resets_only_after_a_stable_connection() {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::{accept_async, tungstenite::Message};

    const HELLO: &str = r#"{"type":"hello","transport":"websocket","session_id":"s-1","audio_params":{"format":"opus","sample_rate":24000,"channels":1,"frame_duration":60}}"#;
    let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_url = format!("ws://{}/voice/ws/", ws_listener.local_addr().unwrap());
    tokio::spawn(async move {
        // Connections 0 and 1 drop right after hello; 2 stays up past the threshold.
        for connection in 0.. {
            let Ok((stream, _)) = ws_listener.accept().await else {
                return;
            };
            let mut socket = accept_async(stream).await.unwrap();
            let _ = socket.next().await;
            socket.send(Message::Text(HELLO.into())).await.unwrap();
            if connection == 2 {
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
            let _ = socket.close(None).await;
        }
    });
    let ota_url = super::support::ota_for_every(&ws_url).await;
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("stable-device", "stable-client"),
        tools: Vec::new(),
    };
    let timeouts = crate::connection::Timeouts {
        stable_after: Duration::from_millis(100),
        ..crate::connection::Timeouts::NONE
    };
    // Report the nominal backoff but sleep only a thousandth of it.
    let (_commands, mut events) =
        crate::spawn_with(held_config(config), |nominal| nominal / 1000, timeouts);

    let mut retries = Vec::new();
    while retries.len() < 4 {
        if let Some(VoiceEvent::Connection(crate::ConnectionState::Offline { retry_in })) =
            tokio::time::timeout(Duration::from_secs(5), events.recv())
                .await
                .unwrap()
        {
            retries.push(retry_in.as_millis());
        }
    }
    assert_eq!(retries, [1, 2, 1, 2]);
}

fn held_config(config: VoiceConfig) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    let (sender, receiver) = tokio::sync::watch::channel(Some(config));
    tokio::spawn(async move {
        sender.closed().await;
    });
    receiver
}
