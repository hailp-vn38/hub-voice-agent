use super::rules::Repeat;
use crate::features::tool_args::{definition, error, parse_value};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;
use voice_client::{ToolDef, ToolError};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Create {
    pub label: String,
    /// Date YYYY-MM-DD in the calendar that `repeat` names.
    pub date: String,
    pub repeat: Repeat,
    /// The lunar date is in a leap month.
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
pub fn parse(name: &str, args: Value) -> Result<Request, ToolError> {
    match name {
        "hub_countdown_create" => parse_value(args).map(Request::Create),
        "hub_countdown_remove" => parse_value(args).map(Request::Remove),
        "hub_countdown_list" => parse_value::<List>(args).map(|_| Request::List),
        _ => Err(error("Unknown countdown tool")),
    }
}
