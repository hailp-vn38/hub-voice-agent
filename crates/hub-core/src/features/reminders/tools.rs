use super::rules::ReminderRepeat;
use crate::features::tool_args::{definition, error, parse_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub label: String,
    /// When to remind: RFC 3339 (`2026-11-23T09:00:00+07:00`) or local time
    /// without offset (`2026-11-23T09:00:00`).
    pub at: String,
    pub repeat: ReminderRepeat,
    /// Weekdays for `weekly`, 1 = Monday … 7 = Sunday.
    #[serde(default)]
    pub weekdays: Vec<u8>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Label {
    pub label: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Snooze {
    pub minutes: u64,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {}

pub enum Tool {
    Create(Create),
    List,
    Cancel(Label),
    Snooze(Snooze),
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<Create>("hub_reminder_create", "Create a local reminder"),
        definition::<List>("hub_reminder_list", "List local reminders"),
        definition::<Label>("hub_reminder_cancel", "Cancel a reminder by label"),
        definition::<Snooze>("hub_reminder_snooze", "Snooze the active reminder"),
    ]
}
pub fn parse(name: &str, args: Value) -> Result<Tool, ToolError> {
    match name {
        "hub_reminder_create" => parse_value(args).map(Tool::Create),
        "hub_reminder_list" => parse_value::<List>(args).map(|_| Tool::List),
        "hub_reminder_cancel" => parse_value(args).map(Tool::Cancel),
        "hub_reminder_snooze" => parse_value(args).map(Tool::Snooze),
        _ => Err(error("Unknown reminder tool")),
    }
}
