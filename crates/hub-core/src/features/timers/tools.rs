use crate::features::tool_args::{definition, error, parse_value};
use schemars::JsonSchema;
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
pub fn parse(name: &str, args: Value) -> Result<TimerTool, ToolError> {
    match name {
        "hub_timer_start" => parse_value(args).map(TimerTool::Start),
        "hub_timer_adjust" => parse_value(args).map(TimerTool::Adjust),
        "hub_timer_cancel" => parse_value(args).map(TimerTool::Cancel),
        "hub_timer_list" => parse_value::<List>(args).map(|_| TimerTool::List),
        _ => Err(error("Unknown timer tool")),
    }
}
