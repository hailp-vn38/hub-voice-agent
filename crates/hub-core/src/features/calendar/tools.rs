use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub location: Option<String>,
    pub summary: String,
    pub start: String,
    pub end: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {
    pub focus: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<List>(
            "hub_calendar_get",
            "List calendar events; start and end bound the range as RFC3339 timestamps or YYYY-MM-DD dates",
        ),
        definition::<Create>(
            "hub_calendar_create",
            "Create an event on the user's Google Calendar (\"thêm lịch họp 3 giờ chiều mai\"); start and end are RFC3339 timestamps or all-day YYYY-MM-DD dates (end exclusive). For \"nhắc tôi ...\" use hub_reminder_create",
        ),
    ]
}
