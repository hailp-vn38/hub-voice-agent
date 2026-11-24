use super::rules::ReminderRepeat;
use crate::features::tool_args::{definition, error, parse_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    /// What to remind; the hub speaks it when due, e.g. "uống thuốc".
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
    /// Minutes to snooze.
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
        definition::<Create>(
            "hub_reminder_create",
            "Remind the user of something at a clock time, once, daily or weekly; when \
             due the hub shows an alert and speaks the label (\"nhắc tôi 9 giờ gọi mẹ\", \
             \"báo thức 6 giờ sáng\", \"mỗi tối 9h nhắc uống thuốc\"). For \
             \"nhắc tôi sau 20 phút ...\" set at to now plus 20 minutes. Not for a plain \
             duration (hub_timer_start), days until a date (hub_countdown_create) or an \
             appointment on the calendar (hub_calendar_create)",
        ),
        definition::<List>("hub_reminder_list", "List reminders saved on the hub"),
        definition::<Label>("hub_reminder_cancel", "Cancel a reminder by label"),
        definition::<Snooze>(
            "hub_reminder_snooze",
            "Snooze the reminder that is ringing now (\"nhắc lại sau 5 phút\")",
        ),
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
