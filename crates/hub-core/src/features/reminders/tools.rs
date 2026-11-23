use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub label: String,
    pub at: String,
    pub repeat: String,
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
fn definition<T: JsonSchema>(name: &str, description: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(T)).expect("schema serializes"),
    }
}
pub fn parse(name: &str, args: Value) -> Result<Tool, ToolError> {
    let result = match name {
        "hub_reminder_create" => serde_json::from_value(args).map(Tool::Create),
        "hub_reminder_list" => serde_json::from_value::<List>(args).map(|_| Tool::List),
        "hub_reminder_cancel" => serde_json::from_value(args).map(Tool::Cancel),
        "hub_reminder_snooze" => serde_json::from_value(args).map(Tool::Snooze),
        _ => {
            return Err(crate::features::timers::tools::error(
                "Unknown reminder tool",
            ));
        }
    };
    result.map_err(|e| crate::features::timers::tools::error(&format!("Invalid arguments: {e}")))
}
