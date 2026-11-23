#[allow(dead_code)]
mod support;

use futures_util::{SinkExt, StreamExt};
use hub_core::{Focus, Io, Ports, Screen, Store, UiState, run};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

struct TestSystem(AtomicBool);
impl hub_core::System for TestSystem {
    fn time_synced(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}
use support::TestClock;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[tokio::test(start_paused = true)]
async fn timers_are_saved_reloaded_and_limit_is_reported() {
    let dir = std::env::temp_dir().join(format!(
        "hub-timers-{}-{:?}",
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
    let (audio_tx, mut _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let system = Arc::new(TestSystem(AtomicBool::new(false)));
    let task = tokio::spawn(run(
        config,
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(start.clone())),
            system: system.clone(),
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
    for id in 1..=6 {
        let arguments = if id == 6 {
            json!({"minutes":"invalid","label":"egg"})
        } else {
            json!({"minutes":7,"label":"luộc trứng"})
        };
        let frame = json!({"type":"mcp","payload":{"jsonrpc":"2.0","id":id,"method":"tools/call","params":{"name":"hub_timer_start","arguments":arguments}}});
        socket
            .send(Message::Text(frame.to_string().into()))
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
        if id == 1 {
            assert_eq!(ui_rx.borrow().view.screen, Screen::Timers);
            assert_eq!(
                ui_rx.borrow().view.focus,
                Some(Focus::Timer(ui_rx.borrow().timers[0].id))
            );
        }
        if id <= 4 {
            assert!(reply["payload"]["result"].is_object(), "{reply}");
        } else if id == 5 {
            assert!(
                reply["payload"]["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains('4')
            );
        } else {
            assert!(
                reply["payload"]["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("Invalid arguments")
            );
        }
    }
    assert_eq!(ui_rx.borrow().timers.len(), 4);
    tokio::time::advance(Duration::from_secs(420)).await;
    tokio::task::yield_now().await;
    assert!(ui_rx.borrow().alert.is_none());
    assert!(_audio_rx.try_recv().is_err());
    system.0.store(true, Ordering::SeqCst);
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        _audio_rx.recv().await,
        Some(hub_core::AudioCommand::Play(hub_core::Sound::TimerChime))
    );
    let first = ui_rx.borrow().alert.clone().unwrap();
    tokio::time::advance(Duration::from_secs(120)).await;
    assert_eq!(
        _audio_rx.recv().await,
        Some(hub_core::AudioCommand::StopSound(
            hub_core::Sound::TimerChime
        ))
    );
    assert_eq!(
        _audio_rx.recv().await,
        Some(hub_core::AudioCommand::Play(hub_core::Sound::TimerChime))
    );
    assert_ne!(ui_rx.borrow().alert.as_ref(), Some(&first));
    input_tx
        .send(hub_core::UserInput::DismissAlert)
        .await
        .unwrap();
    assert_eq!(
        _audio_rx.recv().await,
        Some(hub_core::AudioCommand::StopSound(
            hub_core::Sound::TimerChime
        ))
    );
    assert!(ui_rx.borrow().alert.is_none());
    drop(ui_rx);
    task.await.unwrap().unwrap();
    let (voice_tx, _commands) = mpsc::channel(1);
    let (_events, voice_rx) = mpsc::channel(1);
    let (audio_tx, _audio) = mpsc::channel(1);
    let (_audio_events, audio_rx) = mpsc::channel(1);
    let (_input, input_rx) = mpsc::channel(1);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let config = hub_core::ConfigStore::open(&dir).unwrap();
    let task = tokio::spawn(run(
        config,
        Store::open(&dir).unwrap(),
        Ports {
            clock: Arc::new(TestClock::at(start)),
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
    while ui_rx.borrow_and_update().timers.len() != 2 {
        ui_rx.changed().await.unwrap();
    }
    assert_eq!(ui_rx.borrow().timers.len(), 2);
    assert_eq!(ui_rx.borrow().timers[0].label, "luộc trứng");
    drop(ui_rx);
    task.await.unwrap().unwrap();
    drop(config_tx);
    std::fs::remove_dir_all(dir).unwrap();
}
