use hub::voice_settings::VoiceSettings;

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
    assert_eq!(
        reopened.ota_url().unwrap().as_str(),
        "http://localhost:8080/voice/ota"
    );
}

#[tokio::test]
async fn settings_connects_after_activation_and_shows_ready() {
    use futures_util::{SinkExt, StreamExt};
    use hub::voice_settings::{SettingsCommand, SettingsStatus, start};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use tokio_tungstenite::{accept_async, tungstenite::Message};

    let ws = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_address = ws.local_addr().unwrap();
    tokio::spawn(async move {
        let (stream, _) = ws.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Text(_)))));
        socket.send(Message::Text(serde_json::json!({"type":"hello","transport":"websocket","session_id":"s1","audio_params":{"format":"opus","sample_rate":24000,"channels":1,"frame_duration":60}}).to_string().into())).await.unwrap();
        while socket.next().await.is_some() {}
    });
    let ota = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = ota.local_addr().unwrap();
    tokio::spawn(async move {
        for index in 0..2 {
            let (mut stream, _) = ota.accept().await.unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).await.unwrap() > 0);
            let body = if index == 0 { serde_json::json!({"activation":{"code":"ABC-123"}}) } else { serde_json::json!({"websocket":{"url":format!("ws://{ws_address}"),"token":"token"}}) }.to_string();
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        }
    });
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    settings
        .save_url(&format!("http://{address}/voice/ota"))
        .unwrap();
    let (commands, mut state) = start(settings);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if state.borrow().activation.as_deref() == Some("ABC-123") {
                break;
            }
            state.changed().await.unwrap();
        }
        assert_eq!(state.borrow().status, SettingsStatus::Enrolling);
        commands.send(SettingsCommand::Connect).unwrap();
        loop {
            if state.borrow().status == SettingsStatus::Ready {
                break;
            }
            state.changed().await.unwrap();
        }
        assert_eq!(state.borrow().activation, None);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn saving_url_starts_voice_and_invalid_edit_preserves_config() {
    use hub::voice_settings::{SettingsCommand, SettingsStatus, start};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (request_tx, request_rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut request = [0; 4096];
        let count = stream.read(&mut request).await.unwrap();
        request_tx
            .send(String::from_utf8_lossy(&request[..count]).into_owned())
            .unwrap();
        let body = r#"{"activation":{"code":"SAVE-123"}}"#;
        stream
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .as_bytes(),
            )
            .await
            .unwrap();
    });
    let dir = tempfile::tempdir().unwrap();
    let settings = VoiceSettings::open(dir.path()).unwrap();
    let expected_device = settings.device_id().to_string();
    let (commands, mut state) = start(settings);
    let url = format!("http://{address}/voice/ota");
    commands
        .send(SettingsCommand::SaveUrl(url.clone()))
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let request = request_rx.await.unwrap().to_lowercase();
        assert!(request.contains(&format!("device-id: {expected_device}")));
        loop {
            if state.borrow().activation.as_deref() == Some("SAVE-123") {
                break;
            }
            state.changed().await.unwrap();
        }
        commands
            .send(SettingsCommand::SaveUrl("not a url".into()))
            .unwrap();
        loop {
            if state.borrow().error.is_some() {
                break;
            }
            state.changed().await.unwrap();
        }
        assert_eq!(state.borrow().status, SettingsStatus::Enrolling);
        assert_eq!(state.borrow().ota_url, url);
        assert_eq!(
            VoiceSettings::open(dir.path())
                .unwrap()
                .ota_url()
                .unwrap()
                .as_str(),
            url
        );
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn saving_a_different_url_reconnects_an_existing_client() {
    use hub::voice_settings::{SettingsCommand, start};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
        sync::oneshot,
    };
    async fn server(code: &'static str) -> (String, oneshot::Receiver<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = [0; 4096];
            assert!(stream.read(&mut request).await.unwrap() > 0);
            tx.send(()).unwrap();
            let body = format!(r#"{{"activation":{{"code":"{code}"}}}}"#);
            stream
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });
        (format!("http://{address}/voice/ota"), rx)
    }
    let (first, first_request) = server("first").await;
    let (second, second_request) = server("second").await;
    let dir = tempfile::tempdir().unwrap();
    let mut settings = VoiceSettings::open(dir.path()).unwrap();
    settings.save_url(&first).unwrap();
    let (commands, mut state) = start(settings);
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        first_request.await.unwrap();
        while state.borrow().activation.as_deref() != Some("first") {
            state.changed().await.unwrap();
        }
        commands
            .send(SettingsCommand::SaveUrl(second.clone()))
            .unwrap();
        second_request.await.unwrap();
        while state.borrow().activation.as_deref() != Some("second") {
            state.changed().await.unwrap();
        }
        assert_eq!(
            VoiceSettings::open(dir.path())
                .unwrap()
                .ota_url()
                .unwrap()
                .as_str(),
            second
        );
    })
    .await
    .unwrap();
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
