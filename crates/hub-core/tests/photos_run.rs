#[allow(dead_code)]
mod support;
use futures_util::{SinkExt, StreamExt};
use hub_core::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarQuery, CalendarSettings, CalendarSource,
    Io, NewCalendarEvent, Ports, Screen, Store, UiState, run,
};
use serde_json::{Value, json};
use std::sync::Arc;
use support::TestClock;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
struct FakeCalendar;
impl CalendarSource for FakeCalendar {
    fn list(&self, _: CalendarSettings, _: CalendarQuery) -> CalendarFuture<CalendarDelta> {
        Box::pin(async { Ok(CalendarDelta::default()) })
    }
    fn create(&self, _: CalendarSettings, _: NewCalendarEvent) -> CalendarFuture<CalendarEvent> {
        Box::pin(async { anyhow::bail!("unused") })
    }
}
struct FakeImage;
impl hub_core::ImageFetcher for FakeImage {
    fn fetch(
        &self,
        _: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<hub_core::FetchedImage>> + Send + '_>,
    > {
        Box::pin(async {
            Ok(hub_core::FetchedImage {
                image: vec![1, 2],
                thumbnail: vec![3],
            })
        })
    }
}
#[tokio::test]
async fn show_image_displays_downloaded_photo() {
    let dir = tempfile::tempdir().unwrap();
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
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|c| c.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/")))
        .unwrap();
    let (config_tx, config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
    let (audio_tx, _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let task = tokio::spawn(run(
        config,
        Store::open(dir.path()).unwrap(),
        Ports {
            home: Arc::new(hub_core::HomeAssistant::default()),
            clock: Arc::new(TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: Arc::new(FakeCalendar),
            images: Arc::new(FakeImage),
            system: Arc::new(support::Synced),
            weather: Arc::new(hub_core::OpenMeteo::default()),
        },
        Io {
            voice_config_tx: config_tx,
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
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_show_image","arguments":{"images":[{"url":"https://example.com/a.png","caption":"Lake"},{"url":"https://images.example/b.png","caption":"Mountain"}]}}}}).to_string().into())).await.unwrap();
    let reply: Value = loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 1 {
                break value;
            }
        }
    };
    assert_eq!(reply["payload"]["result"]["loaded"], 2);
    assert_eq!(ui_rx.borrow().view.screen, Screen::Photos);
    assert_eq!(ui_rx.borrow().photos.len(), 2);
    assert!(
        ui_rx
            .borrow()
            .photos
            .iter()
            .any(|photo| photo.caption == "Lake")
    );
    assert!(
        ui_rx
            .borrow()
            .photos
            .iter()
            .any(|photo| photo.source == "images.example")
    );
    assert!(std::path::Path::new(&ui_rx.borrow().photos[0].image_path).exists());
    let id = ui_rx.borrow().photos[0].id.clone();
    input_tx
        .send(hub_core::UserInput::PinPhoto {
            id: id.clone(),
            pinned: true,
        })
        .await
        .unwrap();
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert!(ui_rx.borrow().photos[0].pinned);
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hub_image_list","arguments":{}}}}).to_string().into())).await.unwrap();
    let list: Value = loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 2 {
                break value;
            }
        }
    };
    assert_eq!(
        list["payload"]["result"]["images"][0]["caption"],
        "Mountain"
    );
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"hub_image_control","arguments":{"action":"unpin"}}}}).to_string().into())).await.unwrap();
    let control: Value = loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 3 {
                break value;
            }
        }
    };
    assert_eq!(control["payload"]["result"]["ok"], true);
    assert!(!ui_rx.borrow().photos[0].pinned);

    let photo_to_remove = ui_rx.borrow().photos[1].clone();
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"hub_image_control","arguments":{"action":"next","id":id}}}}).to_string().into())).await.unwrap();
    let next: Value = loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 4 {
                break value;
            }
        }
    };
    assert_eq!(next["payload"]["result"]["id"], photo_to_remove.id);
    assert!(hub_core::present(&ui_rx.borrow()).photos.viewing);
    assert_eq!(hub_core::present(&ui_rx.borrow()).photos.viewer_index, 1);
    input_tx
        .send(hub_core::UserInput::Focus(None))
        .await
        .unwrap();
    while hub_core::present(&ui_rx.borrow()).photos.viewing {
        ui_rx.changed().await.unwrap();
    }
    assert_eq!(ui_rx.borrow().view.screen, Screen::Photos);

    let image_path = photo_to_remove.image_path.clone();
    let thumbnail_path = photo_to_remove.thumbnail_path.clone();
    input_tx
        .send(hub_core::UserInput::RemovePhoto {
            id: photo_to_remove.id,
        })
        .await
        .unwrap();
    while ui_rx.borrow().photos.len() != 1 {
        ui_rx.changed().await.unwrap();
    }
    assert!(!std::path::Path::new(&image_path).exists());
    assert!(!std::path::Path::new(&thumbnail_path).exists());
    task.abort();
}
