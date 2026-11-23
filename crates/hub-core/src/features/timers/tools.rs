use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Start {
    pub minutes: u64,
    pub label: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Adjust {
    pub label: String,
    pub delta_seconds: Option<i64>,
    pub paused: Option<bool>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cancel {
    pub label: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {}

pub enum TimerTool {
    Start(Start),
    Adjust(Adjust),
    Cancel(Cancel),
    List,
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<Start>("hub_timer_start", "Start a timer in minutes"),
        definition::<Adjust>(
            "hub_timer_adjust",
            "Adjust, pause or resume a timer by label",
        ),
        definition::<Cancel>("hub_timer_cancel", "Cancel a timer by label"),
        definition::<List>("hub_timer_list", "List timers"),
    ]
}
fn definition<T: JsonSchema>(name: &str, description: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(T)).expect("schema serializes"),
    }
}
pub fn parse(name: &str, args: Value) -> Result<TimerTool, ToolError> {
    let parsed = match name {
        "hub_timer_start" => serde_json::from_value(args).map(TimerTool::Start),
        "hub_timer_adjust" => serde_json::from_value(args).map(TimerTool::Adjust),
        "hub_timer_cancel" => serde_json::from_value(args).map(TimerTool::Cancel),
        "hub_timer_list" => serde_json::from_value::<List>(args).map(|_| TimerTool::List),
        _ => return Err(error("Unknown timer tool")),
    };
    parsed.map_err(|e| error(&format!("Invalid arguments: {e}")))
}
pub fn error(message: &str) -> ToolError {
    ToolError {
        code: -32602,
        message: message.into(),
        data: None,
    }
}
