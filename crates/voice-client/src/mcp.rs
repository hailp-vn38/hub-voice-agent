use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};

use crate::{ToolCall, ToolDef, ToolError, VoiceEvent};

const TOOLS_PAGE_SIZE: usize = 100;

pub(super) fn discovery_response(payload: &Value, tools: &[ToolDef]) -> Option<Value> {
    let id = payload.get("id")?.as_u64()?;
    match payload.get("method")?.as_str()? {
        "initialize" => Some(json!({
            "jsonrpc": "2.0", "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "voice-client", "version": env!("CARGO_PKG_VERSION") }
            }
        })),
        "tools/list" => {
            let start = match payload.pointer("/params/cursor") {
                None => 0,
                Some(cursor) => match cursor
                    .as_str()
                    .and_then(|cursor| cursor.parse::<usize>().ok())
                {
                    Some(start) if start <= tools.len() => start,
                    _ => {
                        return Some(json!({
                            "jsonrpc": "2.0", "id": id,
                            "error": { "code": -32602, "message": "Invalid tools/list cursor" }
                        }));
                    }
                },
            };
            let end = start.saturating_add(TOOLS_PAGE_SIZE).min(tools.len());
            let page = tools[start..end]
                .iter()
                .map(|tool| {
                    json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.input_schema,
                    })
                })
                .collect::<Vec<_>>();
            let mut result = json!({ "tools": page });
            if end < tools.len() {
                result["nextCursor"] = json!(end.to_string());
            }
            Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
        }
        _ => None,
    }
}

pub(super) fn mcp_payload(value: &Value) -> Option<&Value> {
    (value.get("type")?.as_str()? == "mcp")
        .then(|| value.get("payload"))
        .flatten()
}

pub(super) async fn handle(
    payload: &Value,
    tools: &[ToolDef],
    events: &mpsc::Sender<VoiceEvent>,
    replies: &mpsc::UnboundedSender<(u64, Value)>,
) -> Option<Value> {
    if let Some(response) = discovery_response(payload, tools) {
        return Some(response);
    }
    let id = payload.get("id")?.as_u64()?;
    match payload.get("method")?.as_str()? {
        "tools/call" => {
            let params = payload.get("params").unwrap_or(&Value::Null);
            let name = params.get("name")?.as_str()?.to_owned();
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let (reply_tx, reply_rx) = oneshot::channel::<Result<Value, ToolError>>();
            let replies = replies.clone();
            tokio::spawn(async move {
                let response = match reply_rx.await {
                    Ok(Ok(result)) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
                    Ok(Err(tool_error)) => {
                        let mut error =
                            json!({ "code": tool_error.code, "message": tool_error.message });
                        if let Some(data) = tool_error.data {
                            error["data"] = data;
                        }
                        json!({ "jsonrpc": "2.0", "id": id, "error": error })
                    }
                    Err(_) => error(id, -32603, "Tool call was dropped before a reply"),
                };
                let _ = replies.send((id, response));
            });
            if events
                .send(VoiceEvent::ToolCall(ToolCall {
                    name,
                    arguments,
                    reply: Some(reply_tx),
                }))
                .await
                .is_err()
            {
                return None;
            }
            None
        }
        _ => Some(error(id, -32601, "Method not found")),
    }
}

fn error(id: u64, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

pub(super) fn tool_status(value: &Value) -> Option<VoiceEvent> {
    if value.get("type")?.as_str()? != "tool" {
        return None;
    }
    let name = value.get("name")?.as_str()?.to_owned();
    let state = match value.get("state")?.as_str()? {
        "started" => crate::ToolState::Started,
        "completed" => crate::ToolState::Completed,
        "failed" => crate::ToolState::Failed,
        _ => return None,
    };
    Some(VoiceEvent::ServerTool { name, state })
}
