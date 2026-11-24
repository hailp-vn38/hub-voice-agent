#[allow(dead_code)]
mod support;
use futures_util::{SinkExt, StreamExt};
use hub_core::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarQuery, CalendarSettings, CalendarSource,
    Focus, Io, NewCalendarEvent, PendingState, Ports, Screen, Store, UiState, present, run,
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
struct Fake {
    release_create: Arc<tokio::sync::Notify>,
}
impl CalendarSource for Fake {
    fn list(&self, _: CalendarSettings, _: CalendarQuery) -> CalendarFuture<CalendarDelta> {
        Box::pin(async {
            Ok(CalendarDelta {
                sync_token: Some("token".into()),
                ..Default::default()
            })
        })
    }
    fn create(
        &self,
        _: CalendarSettings,
        event: NewCalendarEvent,
    ) -> CalendarFuture<CalendarEvent> {
        let release_create = self.release_create.clone();
        Box::pin(async move {
            if event.summary == "Fail" {
                anyhow::bail!("offline");
            }
            release_create.notified().await;
            assert_eq!(event.location.as_deref(), Some("Hanoi"));
            Ok(CalendarEvent {
                location: event.location.clone(),
                description: None,
                id: "created".into(),
                summary: event.summary,
                start: event.start,
                end: event.end,
            })
        })
    }
}

async fn wait_for_pending(
    ui_rx: &mut watch::Receiver<UiState>,
    title: &str,
    pending: PendingState,
) {
    let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            if present(&ui_rx.borrow())
                .calendar
                .blocks
                .iter()
                .any(|block| block.title == title && block.pending == pending)
            {
                return;
            }
            ui_rx.changed().await.unwrap();
        }
    })
    .await;
    assert!(
        result.is_ok(),
        "calendar create pending state was not published: {:?}, spawn {}",
        present(&ui_rx.borrow()).calendar,
        ui_rx.borrow().events.spawn
    );
}

#[tokio::test]
async fn creating_event_replies_after_source_and_focuses_event() {
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
    let release_create = Arc::new(tokio::sync::Notify::new());
    let task = tokio::spawn(run(
        config,
        Store::in_memory().unwrap(),
        Ports {
            device: std::sync::Arc::new(hub_core::NoDevice),
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            clock: Arc::new(TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: Arc::new(Fake {
                release_create: release_create.clone(),
            }),
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
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_calendar_create","arguments":{"summary":"Lunch","start":"2026-11-24","end":"2026-11-25","location":"Hanoi"}}}}).to_string().into())).await.unwrap();
    wait_for_pending(&mut ui_rx, "Lunch", PendingState::Saving).await;
    assert_eq!(present(&ui_rx.borrow()).calendar.selected_day, "2026-11-24");
    release_create.notify_one();
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
    wait_for_pending(&mut ui_rx, "Lunch", PendingState::Saved).await;
    assert_eq!(
        reply["payload"]["result"]["structuredContent"]["event"]["id"],
        "created"
    );
    assert_eq!(ui_rx.borrow().view.screen, Screen::Calendar);
    assert_eq!(
        ui_rx.borrow().view.focus,
        Some(Focus::Calendar("created".into()))
    );
    assert_eq!(ui_rx.borrow().calendar[0].summary, "Lunch");
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"hub_calendar_get","arguments":{"focus":"2026-11-24"}}}}).to_string().into())).await.unwrap();
    loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 3 {
                assert_eq!(
                    value["payload"]["result"]["structuredContent"]["events"][0]["id"],
                    "created"
                );
                break;
            }
        }
    }
    assert_eq!(
        ui_rx.borrow().view.focus,
        Some(Focus::Calendar("2026-11-24".into()))
    );
    input_tx
        .send(hub_core::UserInput::Open(Screen::Center))
        .await
        .unwrap();
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hub_calendar_create","arguments":{"summary":"Fail","start":"2026-11-23","end":"2026-11-24"}}}}).to_string().into())).await.unwrap();
    loop {
        let message = tokio::time::timeout(std::time::Duration::from_secs(5), socket.next())
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        if let Message::Text(text) = message {
            let value: Value = serde_json::from_str(&text).unwrap();
            if value["payload"]["id"] == 2 {
                assert!(value["payload"]["error"].is_object());
                break;
            }
        }
    }
    wait_for_pending(&mut ui_rx, "Fail", PendingState::Failed).await;
    assert_eq!(ui_rx.borrow().view.screen, Screen::Calendar);
    task.abort();
}
