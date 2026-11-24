#[allow(dead_code)]
mod support;

use futures_util::{SinkExt, StreamExt};
use hub_core::{Io, Ports, Store, UiState, UserInput, present, run};
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
async fn countdown_updates_at_midnight_through_run() {
    let dir = std::env::temp_dir().join(format!(
        "hub-countdowns-{}-{:?}",
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
        assert!(conn.read(&mut buf).await.unwrap() > 0);
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
    let (audio_tx, _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let task = tokio::spawn(run(
        config,
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(start.clone())),
            calendar: Arc::new(hub_core::GoogleCalendar::default()),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
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
    let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_countdown_create","arguments":{"label":"Tết","date":"2026-11-24","repeat":"none"}}}});
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
    loop {
        let message = socket.next().await.unwrap().unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 1 {
                assert!(value["payload"]["result"]["structuredContent"].is_object(), "{value}");
                break;
            }
        }
    }
    assert_eq!(ui_rx.borrow().countdowns[0].days, 2);
    assert_eq!(ui_rx.borrow().view.screen, hub_core::Screen::Countdowns);
    assert_eq!(
        ui_rx.borrow().view.focus,
        Some(hub_core::Focus::Countdown(ui_rx.borrow().countdowns[0].id))
    );
    for (id, label) in [(2, "Trứng luộc"), (3, "Trứng chiên")] {
        let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"hub_timer_start","arguments":{"label":label,"minutes":1}}}});
        socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .unwrap();
        loop {
            if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["payload"]["id"] == id {
                    assert!(value["payload"]["result"]["structuredContent"].is_object());
                    break;
                }
            }
        }
    }
    let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"hub_timer_cancel","arguments":{"label":"trung"}}}});
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 4 {
                let error = value["payload"]["error"]["message"].as_str().unwrap();
                assert!(error.contains("Trứng luộc") && error.contains("Trứng chiên"));
                break;
            }
        }
    }
    let create = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"hub_countdown_create","arguments":{"label":"Trip","date":"2026-11-28","repeat":"none"}}}});
    socket
        .send(Message::Text(create.to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 5 {
                assert!(value["payload"]["result"]["structuredContent"].is_object(), "{value}");
                break;
            }
        }
    }
    let trip_id = ui_rx
        .borrow()
        .countdowns
        .iter()
        .find(|countdown| countdown.label == "Trip")
        .unwrap()
        .id;
    assert_eq!(present(&ui_rx.borrow()).countdowns.main.label, "Tết");

    tokio::time::advance(Duration::from_secs(12 * 3600)).await;
    tokio::task::yield_now().await;
    assert_eq!(ui_rx.borrow().countdowns[0].days, 1);
    tokio::time::advance(Duration::from_secs(24 * 3600)).await;
    tokio::task::yield_now().await;
    assert_eq!(ui_rx.borrow().countdowns[0].days, 0);
    assert_eq!(ui_rx.borrow().events.confetti, 1);
    assert_eq!(present(&ui_rx.borrow()).countdowns.confetti_serial, 1);
    input_tx
        .send(UserInput::RemoveCountdown(trip_id))
        .await
        .unwrap();
    while ui_rx
        .borrow()
        .countdowns
        .iter()
        .any(|countdown| countdown.id == trip_id)
    {
        ui_rx.changed().await.unwrap();
    }
    assert_eq!(ui_rx.borrow().countdowns.len(), 1);
    assert_eq!(ui_rx.borrow().view.focus, None);
    drop(ui_rx);
    task.await.unwrap().unwrap();
    drop(config_tx);
    let (voice_tx, _commands) = mpsc::channel(1);
    let (_events, voice_rx) = mpsc::channel(1);
    let (audio_tx, _audio) = mpsc::channel(1);
    let (_audio_events, audio_rx) = mpsc::channel(1);
    let (_input, input_rx) = mpsc::channel(1);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let (voice_config_tx, _) = watch::channel(None);
    let restart: jiff::Zoned = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let task = tokio::spawn(run(
        hub_core::ConfigStore::open(&dir).unwrap(),
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(restart)),
            calendar: Arc::new(hub_core::GoogleCalendar::default()),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            system: Arc::new(support::Synced),
        },
        Io {
            voice_config_tx,
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));
    ui_rx.changed().await.unwrap();
    assert_eq!(ui_rx.borrow().countdowns.len(), 1);
    assert_eq!(ui_rx.borrow().countdowns[0].label, "Tết");
    let progress = present(&ui_rx.borrow()).countdowns.main.progress;
    assert!((progress - 2.0 / 3.0).abs() < 0.001, "progress={progress}");
    drop(ui_rx);
    task.await.unwrap().unwrap();
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test(start_paused = true)]
async fn legacy_countdowns_without_creation_metadata_start_at_zero_progress() {
    let dir = tempfile::tempdir().unwrap();
    drop(Store::open(dir.path()).unwrap());
    let connection = rusqlite::Connection::open(dir.path().join("hub.sqlite3")).unwrap();
    connection
        .execute(
            "INSERT INTO countdowns (id, label, date, repeat, lunar_leap) VALUES (1, 'Hạn nộp', '2026-11-25', 'none', 0)",
            [],
        )
        .unwrap();
    drop(connection);

    let start: jiff::Zoned = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let hub = support::Hub::start_with_store(start, Store::open(dir.path()).unwrap());
    tokio::task::yield_now().await;

    let countdown = present(&hub.ui_rx.borrow()).countdowns.main;
    assert_eq!(countdown.key, "1");
    assert_eq!(countdown.progress, 0.0);
    assert_eq!(countdown.progress_text, "0% chặng đường");

    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}
