#[allow(dead_code)]
mod support;

use futures_util::{SinkExt, StreamExt};
use hub_core::{Io, Ports, Store, UiState, run};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use support::TestClock;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test]
async fn reminder_create_through_run() {
    let dir = std::env::temp_dir().join(format!(
        "hub-reminders-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let ws = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws.local_addr().unwrap();
    let ota = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ota_addr = ota.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut conn, _) = ota.accept().await.unwrap();
        let mut buf = [0; 4096];
        conn.read(&mut buf).await.unwrap();
        let body =
            json!({"websocket":{"url":format!("ws://{ws_addr}"),"token":"token"}}).to_string();
        conn.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
    });
    let mut config = hub_core::ConfigStore::open(&dir).unwrap();
    config
        .update(|c| c.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/")))
        .unwrap();
    let (config_tx, config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
    let (audio_tx, _commands) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let task = tokio::spawn(run(
        config,
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(start.clone())),
            system: Arc::new(support::Synced),
        },
        Io {
            voice_config_tx: config_tx.clone(),
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));
    let (stream, _) = ws.accept().await.unwrap();
    let mut socket = accept_async(stream).await.unwrap();
    socket.next().await.unwrap().unwrap();
    socket.send(Message::Text(json!({"type":"hello","transport":"websocket","session_id":"test","audio_params":{"format":"opus","sample_rate":24000,"channels":1,"frame_duration":60}}).to_string().into())).await.unwrap();
    tokio::time::pause();
    let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_reminder_create","arguments":{"label":"Thuốc","at":"2026-11-22T12:01:00+07:00[Asia/Ho_Chi_Minh]","repeat":"none"}}}});
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
    loop {
        let message = socket.next().await.unwrap().unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 1 {
                assert!(value["payload"]["result"].is_object(), "{value}");
                break;
            }
        }
    }
    assert_eq!(ui_rx.borrow().view.screen, hub_core::Screen::Reminders);
    drop(ui_rx);
    task.await.unwrap().unwrap();
    let start = "2026-11-22T12:00:30+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let mut hub = support::Hub::start_with_store(start, Store::open(&dir).unwrap());
    hub.voice_tx
        .send(voice_client::VoiceEvent::Connection(
            voice_client::ConnectionState::Ready,
        ))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        hub.audio_rx.try_recv(),
        Ok(hub_core::AudioCommand::Play(hub_core::Sound::ReminderChime))
    ));
    assert!(matches!(
        hub.ui_rx.borrow().alert,
        Some(hub_core::Alert::Reminder(_))
    ));
    let request_id = match hub.voice_rx.try_recv().unwrap() {
        voice_client::VoiceCommand::DirectSpeech { request_id, text } => {
            assert_eq!(text, "Thuốc");
            request_id
        }
        other => panic!("unexpected command: {other:?}"),
    };
    hub.voice_tx
        .send(voice_client::VoiceEvent::DirectSpeechRejected {
            request_id: request_id.clone(),
            reason: voice_client::DirectSpeechError::SessionBusy,
        })
        .await
        .unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;
    assert!(
        matches!(hub.voice_rx.try_recv(), Ok(voice_client::VoiceCommand::DirectSpeech { request_id: id, .. }) if id == request_id)
    );
    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
    let start = "2026-11-22T12:00:30+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let mut offline = support::Hub::start_with_store(start, Store::open(&dir).unwrap());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(31)).await;
    tokio::task::yield_now().await;
    assert!(matches!(
        offline.audio_rx.try_recv(),
        Ok(hub_core::AudioCommand::Play(hub_core::Sound::ReminderChime))
    ));
    assert!(offline.voice_rx.try_recv().is_err());
    drop(offline.ui_rx);
    offline.task.await.unwrap().unwrap();
    let start = "2026-11-22T14:01:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let mut missed = support::Hub::start_with_store(start, Store::open(&dir).unwrap());
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert!(missed.audio_rx.try_recv().is_err());
    assert!(missed.ui_rx.borrow().reminder_history[0].missed);
    drop(missed.ui_rx);
    missed.task.await.unwrap().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}
