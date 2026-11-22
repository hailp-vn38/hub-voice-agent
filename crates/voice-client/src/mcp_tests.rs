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

#[tokio::test]
async fn initialize_and_paginated_tools_list_return_configured_tools() {
    let mut server = FakeServer::start().await;
    let tools = (0..101)
        .map(|index| ToolDef {
            name: format!("tool_{index}"),
            description: format!("Tool {index}"),
            input_schema: json!({ "type": "object", "properties": {} }),
        })
        .collect::<Vec<_>>();
    let (_commands, mut events) = crate::spawn(config(&server, tools));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": 10, "method": "initialize", "params": {} }),
        ))
        .await
        .unwrap();
    let initialize = next(&mut connection).await;
    assert_eq!(initialize["payload"]["id"], 10);
    assert_eq!(
        initialize["payload"]["result"]["protocolVersion"],
        "2024-11-05"
    );

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": 11, "method": "tools/list" }),
        ))
        .await
        .unwrap();
    let first_page = next(&mut connection).await;
    assert_eq!(first_page["payload"]["id"], 11);
    assert_eq!(
        first_page["payload"]["result"]["tools"]
            .as_array()
            .unwrap()
            .len(),
        100
    );
    assert_eq!(first_page["payload"]["result"]["nextCursor"], "100");

    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": 12, "method": "tools/list", "params": { "cursor": "100" } }))).await.unwrap();
    let second_page = next(&mut connection).await;
    assert_eq!(second_page["payload"]["id"], 12);
    assert_eq!(
        second_page["payload"]["result"]["tools"][0]["name"],
        "tool_100"
    );
    assert!(second_page["payload"]["result"].get("nextCursor").is_none());
}

#[tokio::test]
async fn tools_call_reply_and_drop_are_correlated_to_numeric_request_ids() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": 42, "method": "tools/call", "params": { "name": "hub_volume", "arguments": { "level": 5 } } }))).await.unwrap();
    let Some(VoiceEvent::ToolCall(call)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("expected tool call")
    };
    assert_eq!(call.name, "hub_volume");
    assert_eq!(call.arguments, json!({ "level": 5 }));
    call.reply(Ok(json!({ "ok": true })));
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], 42);
    assert_eq!(reply["payload"]["result"], json!({ "ok": true }));

    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": 43, "method": "tools/call", "params": { "name": "hub_fail", "arguments": {} } }))).await.unwrap();
    let Some(VoiceEvent::ToolCall(call)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("expected tool call")
    };
    call.reply(Err(ToolError {
        code: 7,
        message: "failed".into(),
        data: Some(json!({ "reason": "device" })),
    }));
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], 43);
    assert_eq!(reply["payload"]["error"]["code"], 7);

    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": 44, "method": "tools/call", "params": { "name": "hub_drop", "arguments": {} } }))).await.unwrap();
    let Some(VoiceEvent::ToolCall(call)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("expected tool call")
    };
    drop(call);
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], 44);
    assert_eq!(reply["payload"]["error"]["code"], -32603);
}

#[tokio::test]
async fn unknown_methods_and_tool_status_frames_reach_the_public_event_seam() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": 5, "method": "unknown/method" }),
        ))
        .await
        .unwrap();
    let response = next(&mut connection).await;
    assert_eq!(response["payload"]["id"], 5);
    assert_eq!(response["payload"]["error"]["code"], -32601);

    connection
        .outgoing
        .send(json!({ "type": "tool", "name": "server_search", "state": "started" }))
        .await
        .unwrap();
    connection
        .outgoing
        .send(json!({ "type": "tool", "name": "server_search", "state": "completed" }))
        .await
        .unwrap();
    connection
        .outgoing
        .send(json!({ "type": "tool", "name": "server_search", "state": "failed" }))
        .await
        .unwrap();
    assert!(
        matches!(timeout(Duration::from_secs(2), events.recv()).await.unwrap(), Some(VoiceEvent::ServerTool { name, state: ToolState::Started }) if name == "server_search")
    );
    assert!(
        matches!(timeout(Duration::from_secs(2), events.recv()).await.unwrap(), Some(VoiceEvent::ServerTool { name, state: ToolState::Completed }) if name == "server_search")
    );
    assert!(
        matches!(timeout(Duration::from_secs(2), events.recv()).await.unwrap(), Some(VoiceEvent::ServerTool { name, state: ToolState::Failed }) if name == "server_search")
    );
}

#[tokio::test]
async fn reply_from_disconnected_socket_is_not_written_after_reconnect() {
    let mut server = FakeServer::start().await;
    let initial = VoiceConfig {
        ota_url: server.ota_url.clone(),
        device: DeviceIdentity::new("mcp-device", "mcp-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(initial);
    let (_commands, mut events) = crate::spawn(config_rx);
    ready(&mut events).await;
    let old_connection = server.connection().await;

    old_connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": 90, "method": "tools/call", "params": { "name": "hub_late", "arguments": {} } }))).await.unwrap();
    let Some(VoiceEvent::ToolCall(old_call)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("expected tool call")
    };
    config_tx.send_modify(|config| {
        config.tools.push(ToolDef {
            name: "changed".into(),
            description: "reconnect trigger".into(),
            input_schema: json!({ "type": "object" }),
        })
    });

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
    let mut new_connection = server.connection().await;
    old_call.reply(Ok(json!({ "late": true })));
    assert!(
        timeout(Duration::from_millis(200), new_connection.incoming.recv())
            .await
            .is_err()
    );
}
