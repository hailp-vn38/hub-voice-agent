use serde_json::json;
use tokio::sync::{mpsc, watch};
use url::Url;

use crate::{
    ConnectionState, DeviceIdentity, DirectSpeechError, TtsOrigin, TtsStopReason, VoiceCommand,
    VoiceConfig, VoiceEvent,
};

#[tokio::test]
async fn direct_speech_uses_ready_session_and_returns_tts_events() {
    let server = DirectSpeechServer::start(Reply::Accepted).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    wait_ready(&mut events).await;

    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "rem-42".into(),
            text: "Đến giờ uống thuốc".into(),
        })
        .await
        .unwrap();

    assert_eq!(
        server.received().await,
        json!({
            "type": "direct_speech", "session_id": "session-direct", "request_id": "rem-42",
            "text": "Đến giờ uống thuốc"
        })
    );
    assert!(matches!(events.recv().await, Some(VoiceEvent::TtsStart {
        origin: TtsOrigin::ClientDirect { request_id }
    }) if request_id == "rem-42"));
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsText { text }) if text == "Đã đến giờ")
    );
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed
        })
    ));
    // This later frame confirms the preceding accepted result has been consumed.
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::Stt { text }) if text == "result-processed")
    );
    assert!(matches!(
        events.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn direct_speech_rejection_maps_server_reason() {
    let server = DirectSpeechServer::start(Reply::Rejected("session_busy")).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    wait_ready(&mut events).await;
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "busy-1".into(),
            text: "Đọc sau".into(),
        })
        .await
        .unwrap();

    let _ = server.received().await;
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected {
        request_id, reason: DirectSpeechError::SessionBusy
    }) if request_id == "busy-1")
    );
}

#[tokio::test]
async fn unrelated_tts_result_does_not_emit_direct_speech_rejection() {
    let server = DirectSpeechServer::start(Reply::UnrelatedThenRejected("session_busy")).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    wait_ready(&mut events).await;
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "busy-1".into(),
            text: "Đọc sau".into(),
        })
        .await
        .unwrap();

    let _ = server.received().await;
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected {
        request_id, reason: DirectSpeechError::SessionBusy
    }) if request_id == "busy-1")
    );
    assert!(matches!(
        events.try_recv(),
        Err(mpsc::error::TryRecvError::Empty)
    ));
}

#[tokio::test]
async fn oversized_text_is_rejected_locally_without_websocket_send() {
    let server = DirectSpeechServer::start(Reply::Silent).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    wait_ready(&mut events).await;
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "large-1".into(),
            text: "x".repeat(1_025),
        })
        .await
        .unwrap();

    assert!(
        matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected {
        request_id, reason: DirectSpeechError::InvalidText
    }) if request_id == "large-1")
    );
    assert!(server.received_nothing().await);
}

#[tokio::test]
async fn invalid_request_id_and_text_are_rejected_without_sending() {
    let server = DirectSpeechServer::start(Reply::Silent).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    wait_ready(&mut events).await;

    for (request_id, text, expected) in [
        ("bad id", "valid text", DirectSpeechError::InvalidRequestId),
        ("valid-id", " \n\t ", DirectSpeechError::InvalidText),
        (
            "valid-id",
            "bad\u{0007}text",
            DirectSpeechError::InvalidText,
        ),
    ] {
        commands
            .send(VoiceCommand::DirectSpeech {
                request_id: request_id.into(),
                text: text.into(),
            })
            .await
            .unwrap();
        assert!(
            matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected {
            reason, ..
        }) if reason == expected)
        );
    }
    assert!(server.received_nothing().await);
}

#[tokio::test]
async fn direct_speech_before_ready_is_rejected_offline() {
    let server = DirectSpeechServer::start(Reply::Silent).await;
    let (commands, mut events) = spawn_client(&server.ota_url);
    commands
        .send(VoiceCommand::DirectSpeech {
            request_id: "offline-1".into(),
            text: "Chào bạn".into(),
        })
        .await
        .unwrap();
    let mut rejected_offline = false;
    for _ in 0..3 {
        if matches!(events.recv().await, Some(VoiceEvent::DirectSpeechRejected {
            request_id, reason: DirectSpeechError::Offline
        }) if request_id == "offline-1")
        {
            rejected_offline = true;
            break;
        }
    }
    assert!(rejected_offline);
}

fn spawn_client(ota_url: &str) -> (mpsc::Sender<VoiceCommand>, mpsc::Receiver<VoiceEvent>) {
    let config = VoiceConfig {
        ota_url: Url::parse(ota_url).unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: Vec::new(),
    };
    crate::spawn(watch::channel(config).1)
}

async fn wait_ready(events: &mut mpsc::Receiver<VoiceEvent>) {
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
}

#[path = "voice_tests/support.rs"]
mod support;
use support::{DirectSpeechServer, Reply};
