#[allow(dead_code)]
mod support;

use futures_util::{SinkExt, StreamExt};
use hub_core::{Io, Ports, Store, UiState, run};
use serde_json::{Value, json};
use std::sync::Arc;
use support::{FakeWeather, TestClock};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test]
async fn weather_preview_returns_home_and_uses_cache() {
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
        conn.read(&mut buf).await.unwrap();
        let body =
            json!({"websocket":{"url":format!("ws://{ws_addr}"),"token":"token"}}).to_string();
        conn.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
    });
    let mut config = hub_core::ConfigStore::open(&dir).unwrap();
    config
        .update(|c| c.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/")))
        .unwrap();
    config
        .update(|c| {
            c.weather.latitude = Some(21.0);
            c.weather.longitude = Some(105.0);
        })
        .unwrap();
    let (config_tx, config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
    let (audio_tx, _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let source = Arc::new(FakeWeather::default());
    let task = tokio::spawn(run(
        config,
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(start.clone())),
            weather: source.clone(),
            calendar: Arc::new(hub_core::sources::google_calendar::GoogleCalendar::default()),
            home: Arc::new(hub_core::HomeAssistant::default()),
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
    let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_weather_get","arguments":{"location":"Đà Nẵng"}}}});
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 1 {
                assert!(text.contains("28"), "{text}");
                break;
            }
        }
    }
    assert_eq!(ui_rx.borrow().view.screen, hub_core::Screen::Weather);
    assert_eq!(ui_rx.borrow().weather.location.as_deref(), Some("Đà Nẵng"));
    input_tx.send(hub_core::UserInput::Back).await.unwrap();
    for _ in 0..10 {
        tokio::task::yield_now().await;
    }
    assert_eq!(ui_rx.borrow().view.screen, hub_core::Screen::Center);
    assert_ne!(ui_rx.borrow().weather.location.as_deref(), Some("Đà Nẵng"));
    let before = source.calls.load(std::sync::atomic::Ordering::SeqCst);
    let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hub_weather_get","arguments":{"location":"Đà Nẵng"}}}});
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 2 {
                assert!(text.contains("28"));
                break;
            }
        }
    }
    assert_eq!(
        source.calls.load(std::sync::atomic::Ordering::SeqCst),
        before
    );
    source.fail.store(true, std::sync::atomic::Ordering::SeqCst);
    tokio::time::advance(std::time::Duration::from_secs(1801)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert!(ui_rx.borrow().weather.stale);
    assert_eq!(
        ui_rx.borrow().weather.forecast.as_ref().unwrap()["current"]["temperature_2m"],
        20
    );
    drop(ui_rx);
    task.await.unwrap().unwrap();
}
