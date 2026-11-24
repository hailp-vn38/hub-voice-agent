use super::*;

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
    assert_eq!(reply["payload"]["result"]["structuredContent"], json!({ "ok": true }));
    assert_eq!(
        reply["payload"]["result"]["content"],
        json!([{ "type": "text", "text": "{\"ok\":true}" }])
    );

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
async fn reply_from_disconnected_socket_is_not_written_after_reconnect() {
    let mut server = FakeServer::start().await;
    let initial = VoiceConfig {
        ota_url: server.ota_url.clone(),
        device: DeviceIdentity::new("mcp-device", "mcp-client"),
        tools: Vec::new(),
    };
    let (config_tx, config_rx) = watch::channel(Some(initial));
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
        config.as_mut().unwrap().tools.push(ToolDef {
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
