//! Device MCP: JSON-RPC requests the server sends to the device's own tools.

use serde_json::{Value, json};
use std::sync::Arc;

use tokio::sync::{Semaphore, mpsc, oneshot};

use crate::{ToolCall, ToolDef, ToolError, VoiceEvent};

const TOOLS_PAGE_SIZE: usize = 100;
/// Device tool calls the server may have waiting on a reply at once.
pub(crate) const MAX_IN_FLIGHT_TOOL_CALLS: usize = 16;

/// Replies from device tool calls, bounded by [`MAX_IN_FLIGHT_TOOL_CALLS`].
pub(crate) struct ToolCalls {
    replies: mpsc::Sender<Value>,
    slots: Arc<Semaphore>,
}

impl ToolCalls {
    pub(crate) fn new() -> (Self, mpsc::Receiver<Value>) {
        let (replies, replies_rx) = mpsc::channel(MAX_IN_FLIGHT_TOOL_CALLS);
        let slots = Arc::new(Semaphore::new(MAX_IN_FLIGHT_TOOL_CALLS));
        (Self { replies, slots }, replies_rx)
    }
}

/// A JSON-RPC request id: a number or a string. `None` means a notification,
/// which never gets a reply.
fn request_id(payload: &Value) -> Option<Value> {
    payload
        .get("id")
        .filter(|id| id.is_number() || id.is_string())
        .cloned()
}

pub(crate) fn discovery_response(payload: &Value, tools: &[ToolDef]) -> Option<Value> {
    let id = request_id(payload)?;
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

pub(crate) fn mcp_payload(value: &Value) -> Option<&Value> {
    (value.get("type")?.as_str()? == "mcp")
        .then(|| value.get("payload"))
        .flatten()
}

pub(crate) async fn handle(
    payload: &Value,
    tools: &[ToolDef],
    events: &mpsc::Sender<VoiceEvent>,
    calls: &ToolCalls,
) -> Option<Value> {
    if let Some(response) = discovery_response(payload, tools) {
        return Some(response);
    }
    let id = request_id(payload)?;
    let Some(method) = payload.get("method").and_then(Value::as_str) else {
        return Some(error(&id, -32600, "Invalid Request"));
    };
    match method {
        "tools/call" => {
            let params = payload.get("params").unwrap_or(&Value::Null);
            let Some(name) = params.get("name").and_then(Value::as_str) else {
                return Some(error(&id, -32602, "tools/call requires a string name"));
            };
            let name = name.to_owned();
            let Ok(slot) = calls.slots.clone().try_acquire_owned() else {
                return Some(error(&id, -32000, "Too many tool calls in flight"));
            };
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            let (reply_tx, reply_rx) = oneshot::channel::<Result<Value, ToolError>>();
            let replies = calls.replies.clone();
            tokio::spawn(async move {
                // Held until the reply is queued, so in-flight plus queued stays bounded.
                let _slot = slot;
                let response = match reply_rx.await {
                    Ok(Ok(result)) => {
                        json!({ "jsonrpc": "2.0", "id": id, "result": call_tool_result(result) })
                    }
                    Ok(Err(tool_error)) => {
                        let mut error =
                            json!({ "code": tool_error.code, "message": tool_error.message });
                        if let Some(data) = tool_error.data {
                            error["data"] = data;
                        }
                        json!({ "jsonrpc": "2.0", "id": id, "error": error })
                    }
                    Err(_) => error(&id, -32603, "Tool call was dropped before a reply"),
                };
                let _ = replies.send(response).await;
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
        _ => Some(error(&id, -32601, "Method not found")),
    }
}

/// MCP `CallToolResult`: the server reads `content` text; `structuredContent` keeps the JSON.
fn call_tool_result(result: Value) -> Value {
    json!({
        "content": [{ "type": "text", "text": result.to_string() }],
        "structuredContent": result,
    })
}

fn error(id: &Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
#[path = "mcp_tests/mod.rs"]
mod tests;
