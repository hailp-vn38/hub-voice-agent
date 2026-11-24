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
    ConnectionState, DeviceIdentity, OpusFrame, ToolState, TtsOrigin, TtsStopReason, VoiceCommand,
    VoiceConfig, VoiceEvent,
};

use support::FakeOtaServer;

#[tokio::test]
async fn disconnect_during_tts_emits_one_interrupted_stop_before_offline() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (release_tx, release_rx) = oneshot::channel();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Text(_)))));
        send_hello(&mut socket).await;
        let _ = release_rx.await;
        socket
            .send(Message::Text(
                json!({"type":"tts", "state":"start"}).to_string().into(),
            ))
            .await
            .unwrap();
        socket
            .send(Message::Binary(vec![1, 2, 3].into()))
            .await
            .unwrap();
        socket.close(None).await.unwrap();
    });
    let ota = FakeOtaServer::start(
        200,
        json!({"websocket":{"url":format!("ws://{address}"), "token":"token"}}),
    )
    .await;
    let (_commands, mut events) = voice_client::spawn(config_receiver(ota.url));

    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    release_tx.send(()).unwrap();
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent
        })
    ));
    assert!(
        matches!(events.recv().await, Some(VoiceEvent::TtsAudio(OpusFrame(data))) if data == [1, 2, 3])
    );
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::TtsStop {
            reason: TtsStopReason::Interrupted
        })
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(ConnectionState::Offline { .. }))
    ));
    assert!(
        timeout(Duration::from_millis(100), events.recv())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn mixed_public_events_preserve_server_order_and_abort_stop_completes_tts() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let (reply_tx, reply_rx) = oneshot::channel();
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut socket = accept_async(stream).await.unwrap();
        assert!(matches!(socket.next().await, Some(Ok(Message::Text(_)))));
        send_hello(&mut socket).await;
        for frame in [
            json!({"type":"stt", "text":"xin chào"}),
            json!({"type":"tool", "name":"server_search", "state":"started"}),
            json!({"type":"mcp", "payload":{"jsonrpc":"2.0", "id":42, "method":"tools/call", "params":{"name":"hub_volume", "arguments":{"level":5}}}}),
        ] {
            socket
                .send(Message::Text(frame.to_string().into()))
                .await
                .unwrap();
        }
        let Some(Ok(Message::Text(reply))) = socket.next().await else {
            panic!("expected MCP tool reply")
        };
        let _ = reply_tx.send(serde_json::from_str::<Value>(&reply).unwrap());
        for frame in [
            json!({"type":"tool", "name":"server_search", "state":"completed"}),
            json!({"type":"tts", "state":"start"}),
            json!({"type":"tts", "state":"sentence_start", "text":"Chào bạn"}),
        ] {
            socket
                .send(Message::Text(frame.to_string().into()))
                .await
                .unwrap();
        }
        socket
            .send(Message::Binary(vec![7, 8].into()))
            .await
            .unwrap();
        let Some(Ok(Message::Text(abort))) = socket.next().await else {
            panic!("expected abort command during TTS")
        };
        assert_eq!(
            serde_json::from_str::<Value>(&abort).unwrap()["type"],
            "abort"
        );
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
    let (commands, mut events) = voice_client::spawn(config_receiver(ota.url));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Connecting))
    ));
    assert!(matches!(
        events.recv().await,
        Some(VoiceEvent::Connection(ConnectionState::Ready))
    ));
    assert!(
        matches!(next_event(&mut events).await, VoiceEvent::Stt { text } if text == "xin chào")
    );
    assert!(
        matches!(next_event(&mut events).await, VoiceEvent::ServerTool { name, state: ToolState::Started } if name == "server_search")
    );
    let VoiceEvent::ToolCall(call) = next_event(&mut events).await else {
        panic!("expected public ToolCall event")
    };
    assert_eq!(call.name, "hub_volume");
    assert_eq!(call.arguments, json!({"level":5}));
    call.reply(Ok(json!({"ok":true})));
    let reply = timeout(Duration::from_secs(2), reply_rx)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reply["payload"]["id"], 42);
    assert_eq!(reply["payload"]["result"]["structuredContent"], json!({"ok":true}));
    assert!(
        matches!(next_event(&mut events).await, VoiceEvent::ServerTool { name, state: ToolState::Completed } if name == "server_search")
    );
    assert!(matches!(
        next_event(&mut events).await,
        VoiceEvent::TtsStart {
            origin: TtsOrigin::Agent
        }
    ));
    assert!(
        matches!(next_event(&mut events).await, VoiceEvent::TtsText { text } if text == "Chào bạn")
    );
    assert!(
        matches!(next_event(&mut events).await, VoiceEvent::TtsAudio(OpusFrame(data)) if data == [7, 8])
    );
    commands.send(VoiceCommand::Abort).await.unwrap();
    assert!(matches!(
        next_event(&mut events).await,
        VoiceEvent::TtsStop {
            reason: TtsStopReason::Completed
        }
    ));
}

#[tokio::test]
async fn public_channels_have_the_documented_capacities() {
    let config = VoiceConfig {
        ota_url: Url::parse("http://127.0.0.1:1/voice/ota/").unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: Vec::new(),
    };
    let (commands, events) = voice_client::spawn(held_config(config));
    assert_eq!(commands.max_capacity(), 64);
    assert_eq!(events.max_capacity(), 256);
}

async fn send_hello(socket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
    socket
        .send(Message::Text(
            json!({"type":"hello", "transport":"websocket", "session_id":"invariants", "audio_params":{"format":"opus", "sample_rate":24000, "channels":1, "frame_duration":60}})
                .to_string()
                .into(),
        ))
        .await
        .unwrap();
}

async fn next_event(events: &mut tokio::sync::mpsc::Receiver<VoiceEvent>) -> VoiceEvent {
    timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
        .expect("voice event channel closed")
}

fn config_receiver(url: String) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    let config = VoiceConfig {
        ota_url: Url::parse(&url).unwrap(),
        device: DeviceIdentity::new("device", "client"),
        tools: Vec::new(),
    };
    held_config(config)
}

fn held_config(config: VoiceConfig) -> tokio::sync::watch::Receiver<Option<VoiceConfig>> {
    let (sender, receiver) = tokio::sync::watch::channel(Some(config));
    tokio::spawn(async move {
        sender.closed().await;
    });
    receiver
}
