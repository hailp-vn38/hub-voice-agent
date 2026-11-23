//! Shared helpers for Device MCP tool schemas, arguments and errors.
use schemars::{JsonSchema, schema_for};
use serde::de::DeserializeOwned;
use voice_client::{ToolCall, ToolDef, ToolError};

pub(crate) fn definition<T: JsonSchema>(name: &str, description: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(T)).expect("schema serializes"),
    }
}

/// An invalid-params error (JSON-RPC -32602) for a tool call.
pub(crate) fn error(message: &str) -> ToolError {
    ToolError {
        code: -32602,
        message: message.into(),
        data: None,
    }
}

pub(crate) fn parse<T: DeserializeOwned>(call: &ToolCall) -> Result<T, ToolError> {
    parse_value(call.arguments.clone())
}

pub(crate) fn parse_value<T: DeserializeOwned>(args: serde_json::Value) -> Result<T, ToolError> {
    serde_json::from_value(args).map_err(|e| error(&format!("Invalid arguments: {e}")))
}
