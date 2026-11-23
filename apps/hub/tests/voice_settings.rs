use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use hub::voice_settings::{SettingsCommand, SettingsState, VoiceSettings, start};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use voice_client::ConnectionState;

/// Fake OTA server: answers each request with the next body in `responses`
/// and forwards the raw request text.
async fn ota(responses: Vec<Value>) -> (String, mpsc::UnboundedReceiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (requests_tx, requests) = mpsc::unbounded_channel();
    tokio::spawn(async move {
        let mut responses = responses.into_iter();
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut request = [0; 4096];
            let count = stream.read(&mut request).await.unwrap();
            let _ = requests_tx.send(String::from_utf8_lossy(&request[..count]).into_owned());
            let body = responses.next().unwrap_or_else(|| json!({})).to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    (format!("http://{address}/voice/ota"), requests)
}

/// Fake WebSocket server that completes the hello and holds the session open.
async fn websocket() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Text(_)))));
        let hello = json!({
            "type": "hello",
            "transport": "websocket",
            "session_id": "s1",
            "audio_params": {
                "format": "opus",
                "sample_rate": 24000,
                "channels": 1,
                "frame_duration": 60
            }
        });
        socket
            .send(Message::Text(hello.to_string().into()))
            .await
            .unwrap();
        while socket.next().await.is_some() {}
    });
    format!("ws://{address}")
}

async fn wait_until(
    state: &mut watch::Receiver<SettingsState>,
    done: impl Fn(&SettingsState) -> bool,
) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while !done(&state.borrow_and_update()) {
            state.changed().await.unwrap();
        }
    })
    .await
    .expect("settings state did not reach the expected value");
}

fn saved_url(dir: &tempfile::TempDir) -> String {
    VoiceSettings::open(dir.path())
        .unwrap()
        .ota_url()
        .unwrap()
        .into()
}

#[test]
fn identity_and_url_survive_restart_and_invalid_url_is_not_saved() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    let device = settings.device_id().to_owned();
    let client = settings.client_id().to_owned();
    assert_ne!(device, client);
    assert!(settings.save_url("not a url").is_err());
    assert_eq!(settings.ota_url(), None);
    settings
        .save_url("http://localhost:8080/voice/ota")
        .unwrap();
    drop(settings);
    let reopened = VoiceSettings::open(dir.path()).unwrap();
    assert_eq!(reopened.device_id(), device);
    assert_eq!(reopened.client_id(), client);
    assert_eq!(saved_url(&dir), "http://localhost:8080/voice/ota");
}

#[test]
fn status_labels_match_settings_screen() {
    let mut state = SettingsState {
        ota_url: String::new(),
        device_id: String::new(),
        status: None,
        activation: None,
        error: None,
    };
    let cases = [
        (None, "Chưa cấu hình"),
        (Some(ConnectionState::Connecting), "Đang kết nối"),
        (Some(ConnectionState::Ready), "Đã kết nối"),
        (Some(ConnectionState::Enrolling), "Chờ kích hoạt"),
        (
            Some(ConnectionState::Offline {
                retry_in: Duration::from_secs(4),
            }),
            "Offline (thử lại sau 4 s)",
        ),
    ];
    for (status, label) in cases {
        state.status = status;
        assert_eq!(state.status_label(), label);
    }
    assert!(!state.is_enrolling());
    state.status = Some(ConnectionState::Enrolling);
    assert!(state.is_enrolling());
}

#[tokio::test]
async fn settings_connects_after_activation_and_shows_ready() {
    let ws = websocket().await;
    let (url, _requests) = ota(vec![
        json!({"activation":{"code":"ABC-123"}}),
        json!({"websocket":{"url":ws,"token":"token"}}),
    ])
    .await;
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    settings.save_url(&url).unwrap();
    let (commands, mut state) = start(settings);
    wait_until(&mut state, |s| s.activation.as_deref() == Some("ABC-123")).await;
    assert!(state.borrow().is_enrolling());
    assert_eq!(state.borrow().status_label(), "Chờ kích hoạt");
    commands.send(SettingsCommand::Connect).unwrap();
    wait_until(&mut state, |s| s.status == Some(ConnectionState::Ready)).await;
    assert_eq!(state.borrow().activation, None);
    assert_eq!(state.borrow().status_label(), "Đã kết nối");
}

#[tokio::test]
async fn saving_url_starts_voice_and_invalid_edit_preserves_config() {
    let (url, mut requests) = ota(vec![json!({"activation":{"code":"SAVE-123"}})]).await;
    let dir = tempfile::tempdir().unwrap();
    let settings = VoiceSettings::open(dir.path()).unwrap();
    let expected_device = settings.device_id().to_string();
    let (commands, mut state) = start(settings);
    assert_eq!(state.borrow().status, None);
    commands
        .send(SettingsCommand::SaveUrl(url.clone()))
        .unwrap();
    let request = requests.recv().await.unwrap().to_lowercase();
    assert!(request.contains(&format!("device-id: {expected_device}")));
    wait_until(&mut state, |s| s.activation.as_deref() == Some("SAVE-123")).await;
    commands
        .send(SettingsCommand::SaveUrl("not a url".into()))
        .unwrap();
    wait_until(&mut state, |s| s.error.is_some()).await;
    assert!(state.borrow().is_enrolling());
    assert_eq!(state.borrow().ota_url, url);
    assert_eq!(saved_url(&dir), url);
}

#[tokio::test]
async fn saving_a_different_url_reconnects_an_existing_client() {
    let (first, mut first_requests) = ota(vec![json!({"activation":{"code":"first"}})]).await;
    let (second, mut second_requests) = ota(vec![json!({"activation":{"code":"second"}})]).await;
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    settings.save_url(&first).unwrap();
    let (commands, mut state) = start(settings);
    first_requests.recv().await.unwrap();
    wait_until(&mut state, |s| s.activation.as_deref() == Some("first")).await;
    commands
        .send(SettingsCommand::SaveUrl(second.clone()))
        .unwrap();
    second_requests.recv().await.unwrap();
    wait_until(&mut state, |s| s.activation.as_deref() == Some("second")).await;
    assert_eq!(saved_url(&dir), second);
}

#[test]
fn saving_url_preserves_external_config_edits() {
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    let path = dir.path().join("config.toml");
    let mut document: toml::Table =
        toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    document.insert("weather".into(), toml::Value::String("hanoi".into()));
    document
        .get_mut("voice")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert("other".into(), toml::Value::String("preserve".into()));
    std::fs::write(&path, toml::to_string(&document).unwrap()).unwrap();
    settings.save_url("http://localhost/voice/ota").unwrap();
    let saved: toml::Table = toml::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(saved["weather"].as_str(), Some("hanoi"));
    assert_eq!(saved["voice"]["other"].as_str(), Some("preserve"));
}
