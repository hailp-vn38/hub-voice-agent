#[allow(dead_code)]
mod support;
use futures_util::{SinkExt, StreamExt};
use hub_core::{HomeEntity, HomeEvent, HomeSettings, HomeSource, Io, Ports, Store, UiState, run};
use serde_json::{Value, json};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Default)]
struct FakeHome {
    sender: Mutex<Option<mpsc::Sender<HomeEvent>>>,
    calls: Mutex<Vec<(String, String)>>,
}
impl HomeSource for FakeHome {
    fn subscribe(&self, _: watch::Receiver<HomeSettings>) -> mpsc::Receiver<HomeEvent> {
        let (tx, rx) = mpsc::channel(16);
        *self.sender.lock().unwrap() = Some(tx);
        rx
    }
    fn call_service(
        &self,
        id: String,
        service: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        self.calls.lock().unwrap().push((id, service));
        Box::pin(async { Ok(()) })
    }
}
async fn send(fake: &FakeHome, event: HomeEvent) {
    let tx = loop {
        if let Some(tx) = fake.sender.lock().unwrap().clone() {
            break tx;
        }
        tokio::task::yield_now().await;
    };
    tx.send(event).await.unwrap();
}
#[tokio::test]
async fn tool_calls_service_and_tracks_state_and_disconnect() {
    let dir = tempfile::tempdir().unwrap();
    let ws = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws.local_addr().unwrap();
    let ota = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ota_addr = ota.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut c, _) = ota.accept().await.unwrap();
        let mut buf = [0; 4096];
        assert!(c.read(&mut buf).await.unwrap() > 0);
        let body =
            json!({"websocket":{"url":format!("ws://{ws_addr}"),"token":"token"}}).to_string();
        c.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    });
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|c| c.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/")))
        .unwrap();
    let (config_tx, config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(config_rx);
    let (audio_tx, _audio_commands) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let fake = Arc::new(FakeHome::default());
    let task = tokio::spawn(run(
        config,
        Store::in_memory().unwrap(),
        Ports {
            calendar: Arc::new(hub_core::sources::google_calendar::GoogleCalendar::default()),
            clock: Arc::new(support::TestClock::at(
                "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: fake.clone(),
            images: Arc::new(hub_core::HttpImageFetcher),
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
    send(&fake, HomeEvent::Connected).await;
    let mut entity = HomeEntity {
        id: "light.desk".into(),
        name: "Desk".into(),
        state: "off".into(),
        stale: false,
    };
    send(&fake, HomeEvent::State(entity.clone())).await;
    while ui_rx.borrow().home.is_empty() {
        ui_rx.changed().await.unwrap();
    }
    let call = |id| json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"hub_home_turn_on","arguments":{"entity":"Desk"}}}});
    socket
        .send(Message::Text(call(1).to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let v: Value = serde_json::from_str(&text).unwrap();
            if v["payload"]["id"] == 1 {
                assert!(text.contains("ok"), "{text}");
                break;
            }
        }
    }
    assert_eq!(
        *fake.calls.lock().unwrap(),
        vec![("light.desk".into(), "turn_on".into())]
    );
    entity.state = "on".into();
    send(&fake, HomeEvent::State(entity)).await;
    while ui_rx.borrow().home[0].state != "on" {
        ui_rx.changed().await.unwrap();
    }
    send(&fake, HomeEvent::Disconnected).await;
    while !ui_rx.borrow().home[0].stale {
        ui_rx.changed().await.unwrap();
    }
    socket
        .send(Message::Text(call(2).to_string().into()))
        .await
        .unwrap();
    loop {
        if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
            let v: Value = serde_json::from_str(&text).unwrap();
            if v["payload"]["id"] == 2 {
                assert!(text.contains("disconnected"), "{text}");
                break;
            }
        }
    }
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
    task.abort();
}
