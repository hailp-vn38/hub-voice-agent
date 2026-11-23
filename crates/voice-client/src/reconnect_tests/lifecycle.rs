use std::time::Duration;

use tokio::{sync::watch, time::timeout};

use super::support::ota_for;
use crate::connection::tests::fake_server;
use crate::{DeviceIdentity, VoiceConfig, VoiceEvent, spawn_with_jitter};

#[tokio::test]
async fn dropping_command_sender_keeps_task_alive_until_config_closes() {
    let websocket = fake_server::FakeServer::start(fake_server::ServerAction::HoldOpen).await;
    let ota_url = ota_for(&websocket.url).await;
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("drop-command-device", "drop-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(Some(config));
    let (commands, mut events) = spawn_with_jitter(config_rx, |nominal| nominal);
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));
    drop(commands);
    assert!(
        timeout(Duration::from_millis(50), events.recv())
            .await
            .is_err()
    );
    drop(config_tx);
    assert!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap()
            .is_none()
    );
    timeout(Duration::from_secs(2), websocket.closed)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn dropping_event_receiver_closes_websocket() {
    let websocket = fake_server::FakeServer::start(fake_server::ServerAction::HoldOpen).await;
    let ota_url = ota_for(&websocket.url).await;
    let config = VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("drop-event-device", "drop-client"),
        tools: Vec::new(),
    };
    let (_config_tx, config_rx) = watch::channel(Some(config));
    let (_commands, mut events) = crate::spawn(config_rx);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));
    drop(events);
    timeout(Duration::from_secs(2), websocket.closed)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn different_config_closes_current_socket_and_connects_to_new_server() {
    let first_server = fake_server::FakeServer::start(fake_server::ServerAction::HoldOpen).await;
    let second_server = fake_server::FakeServer::start(fake_server::ServerAction::HoldOpen).await;
    let first_ota = ota_for(&first_server.url).await;
    let second_ota = ota_for(&second_server.url).await;
    let initial = VoiceConfig {
        ota_url: first_ota,
        device: DeviceIdentity::new("change-device", "change-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(Some(initial.clone()));
    let (_commands, mut events) = crate::spawn(config_rx);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));

    config_tx.send(Some(initial.clone())).unwrap();
    tokio::task::yield_now().await;
    assert!(events.try_recv().is_err());
    config_tx
        .send(Some(VoiceConfig {
            ota_url: second_ota,
            ..initial
        }))
        .unwrap();
    first_server.closed.await.unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));
    assert!(second_server.received.await.is_ok());
}
