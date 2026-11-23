#[allow(dead_code)]
mod support;
use futures_util::{SinkExt, StreamExt};
use hub_core::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarSettings, CalendarSource, Focus, Io,
    NewCalendarEvent, Ports, Screen, Store, UiState, run,
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
struct Fake;
impl CalendarSource for Fake {
    fn list(&self, _: CalendarSettings, _: Option<String>) -> CalendarFuture<CalendarDelta> {
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
        Box::pin(async move {
            Ok(CalendarEvent {
                id: "created".into(),
                summary: event.summary,
                start: event.start,
                end: event.end,
            })
        })
    }
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
        conn.read(&mut buf).await.unwrap();
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
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let task = tokio::spawn(run(
        config,
        Store::in_memory().unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: Arc::new(Fake),
            system: Arc::new(support::Synced),
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
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_calendar_create","arguments":{"summary":"Lunch","start":"2026-11-23T12:00:00+07:00","end":"2026-11-23T13:00:00+07:00"}}}}).to_string().into())).await.unwrap();
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
    assert_eq!(reply["payload"]["result"]["event"]["id"], "created");
    assert_eq!(ui_rx.borrow().view.screen, Screen::Calendar);
    assert_eq!(
        ui_rx.borrow().view.focus,
        Some(Focus::Calendar("created".into()))
    );
    assert_eq!(ui_rx.borrow().calendar[0].summary, "Lunch");
    task.abort();
}
