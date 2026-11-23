use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub summary: String,
    pub start: String,
    pub end: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {
    pub start: Option<String>,
    pub end: Option<String>,
}
fn definition<T: JsonSchema>(name: &str, description: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(T)).unwrap(),
    }
}
pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<List>("hub_calendar_list", "List calendar events"),
        definition::<Create>(
            "hub_calendar_create",
            "Create a calendar event; start and end are RFC3339 timestamps",
        ),
    ]
}
