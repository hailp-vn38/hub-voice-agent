//! Fake voice server for Device MCP tests, shared by the test files below.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
    time::timeout,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use url::Url;

use crate::{DeviceIdentity, ToolDef, ToolError, ToolState, VoiceConfig, VoiceEvent};

struct FakeConnection {
    outgoing: mpsc::Sender<Value>,
    incoming: mpsc::Receiver<Value>,
}

struct FakeServer {
    ota_url: Url,
    connections: mpsc::Receiver<FakeConnection>,
}

impl FakeServer {
    async fn start() -> Self {
        let ws_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws_address = ws_listener.local_addr().unwrap();
        let ws_url = format!("ws://{ws_address}");
        let (connections_tx, connections) = mpsc::channel(4);
        tokio::spawn(async move {
            while let Ok((stream, _)) = ws_listener.accept().await {
                let socket = accept_async(stream).await.unwrap();
                let (mut sink, mut stream) = socket.split();
                assert!(matches!(stream.next().await, Some(Ok(Message::Text(_)))));
                sink.send(Message::Text(json!({
                    "type": "hello", "transport": "websocket", "session_id": "mcp-session",
                    "audio_params": { "format": "opus", "sample_rate": 24000, "channels": 1, "frame_duration": 60 }
                }).to_string().into())).await.unwrap();
                let (outgoing_tx, mut outgoing_rx) = mpsc::channel::<Value>(16);
                let (incoming_tx, incoming_rx) = mpsc::channel::<Value>(16);
                if connections_tx
                    .send(FakeConnection {
                        outgoing: outgoing_tx,
                        incoming: incoming_rx,
                    })
                    .await
                    .is_err()
                {
                    return;
                }
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            message = stream.next() => match message {
                                Some(Ok(Message::Text(text))) => {
                                    let value = serde_json::from_str(&text).unwrap();
                                    if incoming_tx.send(value).await.is_err() { return; }
                                }
                                _ => return,
                            },
                            value = outgoing_rx.recv() => match value {
                                Some(value) => if sink.send(Message::Text(value.to_string().into())).await.is_err() { return; },
                                None => return,
                            },
                        }
                    }
                });
            }
        });

        let ota_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ota_address = ota_listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut stream, _)) = ota_listener.accept().await {
                let mut request = vec![0; 4096];
                let _ = stream.read(&mut request).await.unwrap();
                let body = json!({ "websocket": { "url": ws_url, "token": "token" } }).to_string();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
        });
        Self {
            ota_url: Url::parse(&format!("http://{ota_address}/voice/ota/")).unwrap(),
            connections,
        }
    }

    async fn connection(&mut self) -> FakeConnection {
        timeout(Duration::from_secs(2), self.connections.recv())
            .await
            .unwrap()
            .unwrap()
    }
}

fn config(server: &FakeServer, tools: Vec<ToolDef>) -> watch::Receiver<VoiceConfig> {
    watch::channel(VoiceConfig {
        ota_url: server.ota_url.clone(),
        device: DeviceIdentity::new("mcp-device", "mcp-client"),
        tools,
    })
    .1
}

async fn ready(events: &mut mpsc::Receiver<VoiceEvent>) {
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(crate::ConnectionState::Connecting))
    ));
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::Connection(crate::ConnectionState::Ready))
    ));
}

fn mcp(payload: Value) -> Value {
    json!({ "type": "mcp", "session_id": "mcp-session", "payload": payload })
}

async fn next(connection: &mut FakeConnection) -> Value {
    timeout(Duration::from_secs(2), connection.incoming.recv())
        .await
        .unwrap()
        .unwrap()
}

mod calls;
mod discovery;
mod edges;
