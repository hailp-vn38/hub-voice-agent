#[allow(dead_code)]
#[path = "support/fake_server.rs"]
mod support;

use serde_json::{Value, json};
use std::time::Duration;
use support::{FakeServer, ServerAction};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use url::Url;
use voice_client::{
    ConnectionState, DeviceIdentity, DirectSpeechError, VoiceCommand, VoiceConfig, VoiceEvent,
};

async fn ota(responses: Vec<Value>) -> (Url, mpsc::UnboundedReceiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (requests_tx, requests) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut responses = responses.into_iter();
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = [0; 4096];
            let count = stream.read(&mut request).await.unwrap();
            if requests_tx
                .send(String::from_utf8_lossy(&request[..count]).into_owned())
                .is_err()
            {
                break;
            }
            let body = responses.next().unwrap_or_else(|| json!({})).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    (
        Url::parse(&format!("http://{address}/voice/ota")).unwrap(),
        requests,
    )
}

fn config(ota_url: Url) -> VoiceConfig {
    VoiceConfig {
        ota_url,
        device: DeviceIdentity::new("device", "client"),
        tools: vec![],
    }
}

async fn activation(events: &mut mpsc::Receiver<VoiceEvent>, code: &str) {
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Enrolling))
    ));
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::Activation { code: actual }) if actual == code)
    );
}

#[tokio::test]
async fn activation_waits_without_retrying() {
    let (url, mut requests) = ota(vec![json!({"activation":{"code":"A-123"}})]).await;
    let (_config_tx, config_rx) = watch::channel(config(url));
    let (_commands, mut events) = voice_client::spawn(config_rx);
    activation(&mut events, "A-123").await;
    requests.recv().await.unwrap();
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(60)).await;
    tokio::task::yield_now().await;
    assert!(requests.try_recv().is_err());
}

#[tokio::test]
async fn connect_rechecks_activation_then_enters_ready_and_ignores_connect() {
    let websocket = FakeServer::start(ServerAction::HoldOpen).await;
    let (url, mut requests) = ota(vec![
        json!({"activation":{"code":"first"}}),
        json!({"activation":{"code":"second"}}),
        json!({"websocket":{"url":websocket.url,"token":"token"},"activation":{"code":"ignored"}}),
    ])
    .await;
    let (_config_tx, config_rx) = watch::channel(config(url));
    let (commands, mut events) = voice_client::spawn(config_rx);
    activation(&mut events, "first").await;
    requests.recv().await.unwrap();
    commands.send(VoiceCommand::Connect).await.unwrap();
    activation(&mut events, "second").await;
    requests.recv().await.unwrap();
    commands.send(VoiceCommand::Connect).await.unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    requests.recv().await.unwrap();
    commands.send(VoiceCommand::Connect).await.unwrap();
    tokio::time::pause();
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert!(requests.try_recv().is_err());
}

#[tokio::test]
async fn enrolling_rejects_direct_speech_and_reconnects_on_new_config() {
    let (first, mut first_requests) = ota(vec![json!({"activation":{"code":"first"}})]).await;
    let (second, mut second_requests) = ota(vec![json!({"activation":{"code":"new"}})]).await;
    let (config_tx, config_rx) = watch::channel(config(first));
    let (commands, mut events) = voice_client::spawn(config_rx);
    activation(&mut events, "first").await;
    first_requests.recv().await.unwrap();
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "request".into(),
            text: "hello".into(),
        })
        .await
        .unwrap();
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected { request_id, reason: DirectSpeechError::Offline }) if request_id == "request")
    );
    let mut updated = config(second);
    updated.device = DeviceIdentity::new("new-device", "new-client");
    config_tx.send(updated).unwrap();
    activation(&mut events, "new").await;
    let request = second_requests.recv().await.unwrap().to_lowercase();
    assert!(request.contains("device-id: new-device"));
    assert!(request.contains("client-id: new-client"));
}
