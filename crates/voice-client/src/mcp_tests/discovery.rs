use super::*;

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
