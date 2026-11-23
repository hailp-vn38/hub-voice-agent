use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::time::timeout;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[derive(Clone, Copy)]
pub(super) enum Reply {
    Accepted,
    Rejected(&'static str),
    UnrelatedThenRejected(&'static str),
    Silent,
}

pub(super) struct DirectSpeechServer {
    pub(super) ota_url: String,
    received_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<Value>>,
}

impl DirectSpeechServer {
    pub(super) async fn start(reply: Reply) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let (received_tx, received_rx) = mpsc::unbounded_channel();
        let ota_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ota_address = ota_listener.local_addr().unwrap();

        tokio::spawn(async move {
            let (mut stream, _) = ota_listener.accept().await.unwrap();
            let mut request = vec![0; 4096];
            let _ = stream.read(&mut request).await.unwrap();
            let body =
                json!({ "websocket": { "url": format!("ws://{address}"), "token": "token" } })
                    .to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.unwrap();
        });

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut socket = accept_async(stream).await.unwrap();
            let Some(Ok(Message::Text(_))) = socket.next().await else {
                panic!("expected hello")
            };
            socket.send(Message::Text(json!({
                "type": "hello", "transport": "websocket", "session_id": "session-direct",
                "audio_params": { "format": "opus", "sample_rate": 24000, "channels": 1, "frame_duration": 60 }
            }).to_string().into())).await.unwrap();
            let Some(Ok(Message::Text(command))) = socket.next().await else {
                return;
            };
            let command: Value = serde_json::from_str(&command).unwrap();
            let _ = received_tx.send(command);
            match reply {
                Reply::Accepted => {
                    for frame in [
                        json!({ "type": "tts", "session_id": "session-direct", "state": "start", "source": "client_direct", "request_id": "rem-42" }),
                        json!({ "type": "tts", "session_id": "session-direct", "state": "sentence_start", "text": "Đã đến giờ" }),
                        json!({ "type": "tts", "session_id": "session-direct", "state": "stop" }),
                        json!({ "type": "tts", "session_id": "session-direct", "state": "result", "source": "client_direct", "request_id": "rem-42", "outcome": "completed" }),
                        json!({ "type": "stt", "session_id": "session-direct", "text": "result-processed" }),
                    ] {
                        socket
                            .send(Message::Text(frame.to_string().into()))
                            .await
                            .unwrap();
                    }
                }
                Reply::Rejected(reason) | Reply::UnrelatedThenRejected(reason) => {
                    if matches!(reply, Reply::UnrelatedThenRejected(_)) {
                        socket.send(Message::Text(json!({
                            "type": "tts", "session_id": "session-direct", "state": "result",
                            "source": "reminder", "request_id": "rem-elsewhere", "outcome": "rejected", "reason": "session_busy"
                        }).to_string().into())).await.unwrap();
                    }
                    socket.send(Message::Text(json!({
                        "type": "tts", "session_id": "session-direct", "state": "result",
                        "source": "client_direct", "request_id": "busy-1", "outcome": "rejected", "reason": reason
                    }).to_string().into())).await.unwrap();
                }
                Reply::Silent => {}
            }
            while let Some(Ok(message)) = socket.next().await {
                if matches!(message, Message::Close(_)) {
                    break;
                }
            }
        });
        Self {
            ota_url: format!("http://{ota_address}/voice/ota/"),
            received_rx: tokio::sync::Mutex::new(received_rx),
        }
    }

    pub(super) async fn received(&self) -> Value {
        timeout(Duration::from_secs(2), self.received_rx.lock().await.recv())
            .await
            .unwrap()
            .unwrap()
    }

    pub(super) async fn received_nothing(&self) -> bool {
        timeout(
            Duration::from_millis(100),
            self.received_rx.lock().await.recv(),
        )
        .await
        .is_err()
    }
}
