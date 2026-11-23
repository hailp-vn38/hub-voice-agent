use tokio::{
    sync::watch,
    time::{Duration, timeout},
};
use voice_client::{ConnectionState, DirectSpeechError, VoiceCommand, VoiceEvent};

#[tokio::test]
async fn unconfigured_rejects_direct_speech_and_stops_when_config_sender_closes() {
    let (config_tx, config_rx) = watch::channel(None);
    let (commands, mut events) = voice_client::spawn(config_rx);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Unconfigured))
    ));
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "one".into(),
            text: "hello".into(),
        })
        .await
        .unwrap();
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected { request_id, reason: DirectSpeechError::Offline }) if request_id == "one")
    );
    drop(config_tx);
    assert!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap()
            .is_none()
    );
}

#[allow(dead_code)]
#[path = "support/fake_server.rs"]
mod support;

use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use url::Url;
use voice_client::{DeviceIdentity, TtsStopReason, VoiceConfig};

#[tokio::test]
async fn optional_config_connects_once_then_interrupts_tts_and_closes_socket() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (closed_tx, closed_rx) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Text(_)))));
        socket.send(Message::Text(json!({"type":"hello", "transport":"websocket", "session_id":"optional", "audio_params":{"format":"opus", "sample_rate":24000, "channels":1, "frame_duration":60}}).to_string().into())).await.unwrap();
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"start"}).to_string().into(),
            ))
            .await
            .unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Close(_)))));
        closed_tx.send(()).unwrap();
        assert!(
            timeout(Duration::from_millis(150), listener.accept())
                .await
                .is_err()
        );
    });
    let mut ota = support::FakeOtaServer::start(
        200,
        json!({"websocket":{"url":format!("ws://{address}"), "token":"token"}}),
    )
    .await;
    let config = VoiceConfig {
        ota_url: Url::parse(&ota.url).unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: vec![],
    };
    let (config_tx, config_rx) = watch::channel(None);
    let (_commands, mut events) = voice_client::spawn(config_rx);
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Unconfigured))
    ));
    assert!(
        timeout(Duration::from_millis(100), &mut ota.received)
            .await
            .is_err()
    );
    config_tx.send(Some(config.clone())).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStart { .. })
    ));
    config_tx.send(Some(config)).unwrap();
    assert!(
        timeout(Duration::from_millis(100), events.recv())
            .await
            .is_err()
    );
    config_tx.send(None).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Interrupted
        })
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Unconfigured))
    ));
    timeout(Duration::from_secs(2), closed_rx)
        .await
        .unwrap()
        .unwrap();
    server.await.unwrap();
}
