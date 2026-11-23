use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub label: String,
    /// Date YYYY-MM-DD in the requested calendar.
    pub date: String,
    /// none, yearly, or yearly_lunar
    pub repeat: String,
    #[serde(default)]
    pub lunar_leap: bool,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Remove {
    pub label: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {}
pub enum Request {
    Create(Create),
    Remove(Remove),
    List,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<Create>("hub_countdown_create", "Create a countdown"),
        definition::<Remove>("hub_countdown_remove", "Remove a countdown by label"),
        definition::<List>("hub_countdown_list", "List countdowns"),
    ]
}
fn definition<T: JsonSchema>(name: &str, description: &str) -> ToolDef {
    ToolDef {
        name: name.into(),
        description: description.into(),
        input_schema: serde_json::to_value(schema_for!(T)).expect("schema serializes"),
    }
}
pub fn parse(name: &str, args: Value) -> Result<Request, ToolError> {
    let result = match name {
        "hub_countdown_create" => serde_json::from_value(args).map(Request::Create),
        "hub_countdown_remove" => serde_json::from_value(args).map(Request::Remove),
        "hub_countdown_list" => serde_json::from_value::<List>(args).map(|_| Request::List),
        _ => return Err(error("Unknown countdown tool")),
    };
    result.map_err(|e| error(&format!("Invalid arguments: {e}")))
}
pub fn error(message: &str) -> ToolError {
    ToolError {
        code: -32602,
        message: message.into(),
        data: None,
    }
}
