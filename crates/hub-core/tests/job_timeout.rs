#[allow(dead_code)]
mod support;
use futures_util::{SinkExt, StreamExt};
use hub_core::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarQuery, CalendarSettings, CalendarSource,
    Io, NewCalendarEvent, Ports, Store, UiState, run,
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
struct Fake(std::sync::Arc<tokio::sync::Notify>);
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
        let started = self.0.clone();
        let _ = event;
        Box::pin(async move {
            started.notify_one();
            std::future::pending().await
        })
    }
}
#[tokio::test]
async fn timed_out_calendar_job_replies_to_real_tool_call() {
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
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, _ui_rx) = watch::channel(UiState::default());
    let started = Arc::new(tokio::sync::Notify::new());
    let task = tokio::spawn(run(
        config,
        Store::in_memory().unwrap(),
        Ports {
            images: Arc::new(hub_core::HttpImageFetcher),
            home: Arc::new(hub_core::HomeAssistant::default()),
            clock: Arc::new(TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: Arc::new(Fake(started.clone())),
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
    socket.send(Message::Text(json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"hub_calendar_create","arguments":{"summary":"Lunch","start":"2026-11-23T12:00:00+07:00","end":"2026-11-23T13:00:00+07:00"}}}}).to_string().into())).await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), started.notified())
        .await
        .expect("calendar job did not start");
    tokio::time::pause();
    tokio::time::advance(std::time::Duration::from_secs(31)).await;
    tokio::time::resume();
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
    assert_eq!(reply["payload"]["error"]["code"], -32603);
    assert_eq!(reply["payload"]["error"]["message"], "Job timed out");
    task.abort();
}
