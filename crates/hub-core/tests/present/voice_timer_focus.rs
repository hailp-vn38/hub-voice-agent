use futures_util::{SinkExt, StreamExt};
use hub_core::{Focus, Screen, UiState};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test(start_paused = true)]
async fn repeated_timer_start_advances_focus_serial_for_each_focus() {
    let ws = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws.local_addr().unwrap();
    let ota = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ota_addr = ota.local_addr().unwrap();
    let ota_task = tokio::spawn(async move {
        let (mut conn, _) = ota.accept().await.unwrap();
        let mut buf = [0; 4096];
        assert!(conn.read(&mut buf).await.unwrap() > 0);
        let body = json!({
            "websocket": {"url": format!("ws://{ws_addr}"), "token": "token"}
        })
        .to_string();
        conn.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    });

    let dir = tempfile::tempdir().unwrap();
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|settings| {
            settings.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/"));
        })
        .unwrap();
    let (voice_config_tx, voice_config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(voice_config_rx);
    let (audio_tx, _audio_commands) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_sender, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let start = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let task = tokio::spawn(hub_core::run(
        config,
        hub_core::Store::in_memory().unwrap(),
        hub_core::Ports {
            device: std::sync::Arc::new(hub_core::NoDevice),
            clock: Arc::new(crate::support::TestClock::at(start)),
            calendar: Arc::new(hub_core::GoogleCalendar::default()),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            system: Arc::new(crate::support::Synced),
        },
        hub_core::Io {
            voice_config_tx,
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
    socket
        .send(Message::Text(
            json!({
                "type": "hello",
                "transport": "websocket",
                "session_id": "test",
                "audio_params": {
                    "format": "opus",
                    "sample_rate": 24000,
                    "channels": 1,
                    "frame_duration": 60
                }
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

    for id in 1..=2 {
        socket
            .send(Message::Text(
                json!({
                    "type": "mcp",
                    "payload": {
                        "jsonrpc": "2.0",
                        "id": id,
                        "method": "tools/call",
                        "params": {
                            "name": "hub_timer_start",
                            "arguments": {"minutes": 1, "label": "luộc trứng"}
                        }
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let reply: Value = loop {
            let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            if let Message::Text(text) = message {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["payload"]["id"] == id {
                    break value;
                }
            }
        };
        assert!(reply["payload"]["result"].is_object(), "{reply}");
        // On the second call, show replays the old focus before start focuses the new timer.
        let expected_serial = if id == 1 { 1 } else { 3 };
        assert_eq!(ui_rx.borrow().events.focus, expected_serial);
    }

    assert_eq!(ui_rx.borrow().view.screen, Screen::Timers);
    assert_eq!(ui_rx.borrow().timers.len(), 2);
    assert_eq!(ui_rx.borrow().view.focus, Some(Focus::Timer(2)));
    drop(ui_rx);
    task.await.unwrap().unwrap();
    ota_task.await.unwrap();
}
