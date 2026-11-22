use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use url::Url;

use super::support::failing_ota;
use crate::connection_tests::fake_server;
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
    let (_config_tx, config_rx) = watch::channel(config);
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
    let (_config_tx, config_rx) = watch::channel(config);
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
    let (_config_tx, config_rx) = watch::channel(config);
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
