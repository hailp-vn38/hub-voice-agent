use std::time::Duration;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::watch,
};
use url::Url;

use super::support::failing_ota;
use crate::{
    DeviceIdentity, DirectSpeechError, VoiceCommand, VoiceConfig, VoiceEvent, spawn_with_jitter,
};

#[tokio::test(start_paused = true)]
async fn equal_config_does_not_retry_early_but_different_ota_url_retries_immediately() {
    let (first_url, mut first_requests) = failing_ota().await;
    let (second_url, mut second_requests) = failing_ota().await;
    let config = VoiceConfig {
        ota_url: first_url,
        device: DeviceIdentity::new("config-device", "config-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(Some(config.clone()));
    let (_commands, mut events) = spawn_with_jitter(config_rx, |nominal| nominal);

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(first_requests.recv().await.is_some());
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));

    config_tx.send(Some(config)).unwrap();
    tokio::task::yield_now().await;
    assert!(events.try_recv().is_err());
    assert!(first_requests.try_recv().is_err());

    tokio::time::advance(Duration::from_millis(999)).await;
    tokio::task::yield_now().await;
    assert!(events.try_recv().is_err());
    tokio::time::advance(Duration::from_millis(1)).await;
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(first_requests.recv().await.is_some());
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));

    let updated = VoiceConfig {
        ota_url: second_url,
        device: config_tx.borrow().as_ref().unwrap().device.clone(),
        tools: Vec::new(),
    };
    config_tx.send(Some(updated)).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(second_requests.recv().await.is_some());
}

#[tokio::test]
async fn equal_config_during_ota_keeps_the_active_attempt() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (request_seen_tx, request_seen_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = vec![0; 2048];
        let _ = stream.read(&mut request).await.unwrap();
        let _ = request_seen_tx.send(());
        let _ = release_rx.await;
        stream
            .write_all(b"HTTP/1.1 403 Forbidden\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
    });
    let config = VoiceConfig {
        ota_url: Url::parse(&format!("http://{address}/voice/ota/")).unwrap(),
        device: DeviceIdentity::new("equal-ota-device", "equal-ota-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(Some(config.clone()));
    let (commands, mut events) = crate::spawn(config_rx);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    request_seen_rx.await.unwrap();

    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "offline-request".into(),
            text: "hello".into(),
        })
        .await
        .unwrap();
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected { request_id, reason: DirectSpeechError::Offline }) if request_id == "offline-request")
    );

    config_tx.send(Some(config)).unwrap();
    tokio::task::yield_now().await;
    assert!(events.try_recv().is_err());
    release_tx.send(()).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(
            crate::ConnectionState::Offline { .. }
        ))
    ));
}
