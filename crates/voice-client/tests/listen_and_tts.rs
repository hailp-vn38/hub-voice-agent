#[allow(dead_code)]
#[path = "support/fake_server.rs"]
mod support;

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{net::TcpListener, sync::oneshot, time::timeout};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use url::Url;
use voice_client::{
    ConnectionState, DeviceIdentity, OpusFrame, TtsOrigin, TtsStopReason, VoiceCommand,
    VoiceConfig, VoiceEvent,
};

use support::FakeOtaServer;

#[tokio::test]
async fn listen_audio_and_tts_round_trip_preserves_wire_and_event_order() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (received_tx, received_rx) = oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        let Some(Ok(Message::Text(client_hello))) = socket.next().await else {
            panic!("expected ClientHello")
        };
        let client_hello: Value = serde_json::from_str(&client_hello).unwrap();
        socket
            .send(Message::Text(
                json!({
                    "type": "hello",
                    "transport": "websocket",
                    "session_id": "voice-session-5",
                    "audio_params": {"format":"opus", "sample_rate":24000, "channels":1, "frame_duration":60}
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();

        let mut client_frames = Vec::new();
        while client_frames.len() < 6 {
            let Some(Ok(frame)) = socket.next().await else {
                panic!("client closed before sending listen frames")
            };
            client_frames.push(frame);
        }
        let _ = received_tx.send((client_hello, client_frames));

        for frame in [
            json!({"type":"stt", "text":"xin chào", "session_id":"voice-session-5"}).to_string(),
            "{malformed".to_owned(),
            json!({"type":"future_event"}).to_string(),
            json!({"type":"tts", "state":"start", "source":"client_direct", "request_id":"req-7"})
                .to_string(),
            json!({"type":"tts", "state":"sentence_start", "text":"Chào bạn"}).to_string(),
        ] {
            socket.send(Message::Text(frame.into())).await.unwrap();
        }
        socket
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        socket
            .send(Message::Binary(vec![4, 5].into()))
            .await
            .unwrap();
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"stop"}).to_string().into(),
            ))
            .await
            .unwrap();
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"start"}).to_string().into(),
            ))
            .await
            .unwrap();
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"stop"}).to_string().into(),
            ))
            .await
            .unwrap();
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
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));

    commands.send(VoiceCommand::ListenStart).await.unwrap();
    commands
        .send(VoiceCommand::Audio(OpusFrame(vec![10, 11])))
        .await
        .unwrap();
    commands
        .send(VoiceCommand::Audio(OpusFrame(vec![12, 13])))
        .await
        .unwrap();
    commands
        .send(VoiceCommand::Audio(OpusFrame(vec![14, 15])))
        .await
        .unwrap();
    commands.send(VoiceCommand::ListenStop).await.unwrap();
    commands.send(VoiceCommand::Abort).await.unwrap();

    let (hello, frames) = timeout(Duration::from_secs(2), received_rx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(hello["type"], "hello");
    assert_client_text(
        &frames[0],
        json!({"type":"listen", "state":"start", "mode":"realtime", "session_id":"voice-session-5"}),
    );
    assert_binary(&frames[1], &[10, 11]);
    assert_binary(&frames[2], &[12, 13]);
    assert_binary(&frames[3], &[14, 15]);
    assert_client_text(
        &frames[4],
        json!({"type":"listen", "state":"stop", "session_id":"voice-session-5"}),
    );
    assert_client_text(
        &frames[5],
        json!({"type":"abort", "session_id":"voice-session-5"}),
    );

    assert!(matches!(events.recv().await, Some(VoiceEvent::Stt { text }) if text == "xin chào"));
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsStart { origin: TtsOrigin::ClientDirect { request_id } }) if request_id == "req-7")
    );
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsText { text }) if text == "Chào bạn")
    );
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsAudio(OpusFrame(data))) if data == [1, 2, 3])
    );
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsAudio(OpusFrame(data))) if data == [4, 5])
    );
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed
        })
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent
        })
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed
        })
    ));
    server.await.unwrap();
}

fn config_receiver(config: VoiceConfig) -> tokio::sync::watch::Receiver<VoiceConfig> {
    tokio::sync::watch::channel(config).1
}

fn assert_client_text(message: &Message, expected: Value) {
    let Message::Text(text) = message else {
        panic!("expected text frame, got {message:?}")
    };
    let actual: Value = serde_json::from_str(text).unwrap();
    assert_eq!(actual, expected);
}

fn assert_binary(message: &Message, expected: &[u8]) {
    let Message::Binary(binary) = message else {
        panic!("expected binary frame, got {message:?}")
    };
    assert_eq!(binary.as_ref(), expected);
}
