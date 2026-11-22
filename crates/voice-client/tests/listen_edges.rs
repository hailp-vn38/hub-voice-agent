#[allow(dead_code)]
#[path = "support/fake_server.rs"]
mod support;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::{net::TcpListener, sync::oneshot, time::timeout};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use url::Url;
use voice_client::{
    ConnectionState, DeviceIdentity, OpusFrame, TtsStopReason, VoiceCommand, VoiceConfig,
    VoiceEvent,
};

use support::FakeOtaServer;

#[tokio::test]
async fn commands_sent_before_ready_are_discarded() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (hello_tx, hello_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let (frames_tx, frames_rx) = oneshot::channel();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        let Some(Ok(Message::Text(_))) = socket.next().await else {
            panic!("expected ClientHello")
        };
        let _ = hello_tx.send(());
        let _ = release_rx.await;
        socket
            .send(Message::Text(
                json!({
                    "type":"hello", "transport":"websocket", "session_id":"pre-ready",
                    "audio_params":{"format":"opus", "sample_rate":24000, "channels":1, "frame_duration":60}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut frames = Vec::new();
        while let Ok(Some(Ok(frame))) = timeout(Duration::from_millis(20), socket.next()).await {
            frames.push(frame);
        }
        let _ = frames_tx.send(frames);
    });
    let ota = FakeOtaServer::start(
        200,
        json!({"websocket":{"url":format!("ws://{address}"), "token":"token"}}),
    )
    .await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: Vec::new(),
    };
    let (commands, mut events) = voice_client::spawn(config_receiver(config));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    hello_rx.await.unwrap();
    commands.send(VoiceCommand::ListenStart).await.unwrap();
    commands.send(VoiceCommand::ListenStop).await.unwrap();
    commands.send(VoiceCommand::Abort).await.unwrap();
    commands
        .send(VoiceCommand::Audio(OpusFrame(vec![1, 2, 3])))
        .await
        .unwrap();
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "offline-request".to_owned(),
            text: "đọc sau".to_owned(),
        })
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;
    release_tx.send(()).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::DirectSpeechRejected {
            request_id,
            reason: voice_client::DirectSpeechError::Offline
        }) if request_id == "offline-request"
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    assert!(
        timeout(Duration::from_secs(2), frames_rx)
            .await
            .unwrap()
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn audio_events_drop_when_full_and_tts_stop_waits_for_capacity() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (sent_tx, sent_rx) = oneshot::channel();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        let Some(Ok(Message::Text(_))) = socket.next().await else {
            panic!("expected ClientHello")
        };
        socket
            .send(Message::Text(
                json!({
                    "type":"hello", "transport":"websocket", "session_id":"bounded",
                    "audio_params":{"format":"opus", "sample_rate":24000, "channels":1, "frame_duration":60}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        for _ in 0..600 {
            socket
                .send(Message::Binary(vec![7; 60].into()))
                .await
                .unwrap();
        }
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"stop"}).to_string().into(),
            ))
            .await
            .unwrap();
        let _ = sent_tx.send(());
    });
    let ota = FakeOtaServer::start(
        200,
        json!({"websocket":{"url":format!("ws://{address}"), "token":"token"}}),
    )
    .await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: Vec::new(),
    };
    let (_commands, mut events) = voice_client::spawn(config_receiver(config));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    sent_rx.await.unwrap();
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut audio_events = 0;
    let stop_seen = timeout(Duration::from_secs(2), async {
        while let Some(event) = events.recv().await {
            match event {
                VoiceEvent::TtsAudio(_) => audio_events += 1,
                VoiceEvent::TtsStop {
                    reason: TtsStopReason::Completed,
                } => break,
                _ => {}
            }
        }
    })
    .await;
    assert!(
        stop_seen.is_ok(),
        "reader must continue after dropping queued audio"
    );
    assert!(
        audio_events < 600,
        "full event queues must drop some TtsAudio"
    );
}

fn config_receiver(config: VoiceConfig) -> tokio::sync::watch::Receiver<VoiceConfig> {
    tokio::sync::watch::channel(config).1
}
