use std::time::Duration;

use serde_json::json;
use tokio::time::timeout;

use super::{FakeServer, config, mcp, next, ready};
use crate::VoiceEvent;
use crate::mcp::MAX_IN_FLIGHT_TOOL_CALLS;

#[tokio::test]
async fn tools_call_without_name_gets_invalid_params() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": {} }),
        ))
        .await
        .unwrap();
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], 7);
    assert_eq!(reply["payload"]["error"]["code"], -32602);
}

#[tokio::test]
async fn string_request_ids_are_echoed() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": "init-1", "method": "initialize" }),
        ))
        .await
        .unwrap();
    assert_eq!(next(&mut connection).await["payload"]["id"], "init-1");

    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": "call-1", "method": "tools/call", "params": { "name": "hub_volume" } }))).await.unwrap();
    let Some(VoiceEvent::ToolCall(call)) = timeout(Duration::from_secs(2), events.recv())
        .await
        .unwrap()
    else {
        panic!("expected tool call")
    };
    call.reply(Ok(json!({ "ok": true })));
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], "call-1");
    assert_eq!(reply["payload"]["result"]["structuredContent"], json!({ "ok": true }));
}

#[tokio::test]
async fn notifications_get_no_reply() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
        ))
        .await
        .unwrap();
    connection
        .outgoing
        .send(mcp(
            json!({ "jsonrpc": "2.0", "id": 9, "method": "unknown/method" }),
        ))
        .await
        .unwrap();
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], 9);
    assert_eq!(reply["payload"]["error"]["code"], -32601);
}

#[tokio::test]
async fn tool_calls_beyond_the_in_flight_limit_are_refused() {
    let mut server = FakeServer::start().await;
    let (_commands, mut events) = crate::spawn(config(&server, Vec::new()));
    ready(&mut events).await;
    let mut connection = server.connection().await;

    let mut pending = Vec::new();
    for id in 0..MAX_IN_FLIGHT_TOOL_CALLS {
        connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": { "name": "hold" } }))).await.unwrap();
        let Some(VoiceEvent::ToolCall(call)) = timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap()
        else {
            panic!("expected tool call")
        };
        pending.push(call);
    }
    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": "over", "method": "tools/call", "params": { "name": "hold" } }))).await.unwrap();
    let reply = next(&mut connection).await;
    assert_eq!(reply["payload"]["id"], "over");
    assert_eq!(reply["payload"]["error"]["code"], -32000);

    // A finished call frees its slot.
    pending.pop().unwrap().reply(Ok(json!({})));
    assert!(next(&mut connection).await["payload"]["result"]["structuredContent"].is_object());
    connection.outgoing.send(mcp(json!({ "jsonrpc": "2.0", "id": "again", "method": "tools/call", "params": { "name": "hold" } }))).await.unwrap();
    assert!(matches!(
        timeout(Duration::from_secs(2), events.recv())
            .await
            .unwrap(),
        Some(VoiceEvent::ToolCall(_))
    ));
}
