use crate::features::tool_args::{definition, error, parse_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Start {
    /// Duration in whole minutes; convert hours ("1 tiếng rưỡi" = 90).
    pub minutes: u64,
    /// Short name of what is being timed, e.g. "trứng".
    pub label: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Adjust {
    pub label: String,
    /// Seconds to add; negative to subtract.
    pub delta_seconds: Option<i64>,
    /// true pauses, false resumes.
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
        definition::<Start>(
            "hub_timer_start",
            "Start a timer that rings after a duration from now, like a kitchen timer \
             (\"hẹn giờ 7 phút\", \"đếm ngược 10 phút\", \"bấm giờ luộc trứng\"). \
             Use only for a plain duration. For \"nhắc tôi ...\" or a clock time use \
             hub_reminder_create; for days until a date use hub_countdown_create",
        ),
        definition::<Adjust>(
            "hub_timer_adjust",
            "Add or remove time, pause or resume a running timer by label \
             (\"thêm 2 phút\", \"tạm dừng hẹn giờ\")",
        ),
        definition::<Cancel>(
            "hub_timer_cancel",
            "Cancel a running timer by label (\"huỷ hẹn giờ trứng\")",
        ),
        definition::<List>(
            "hub_timer_list",
            "List running timers with time left (\"trứng còn bao lâu?\")",
        ),
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
