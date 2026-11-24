use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Get {
    /// City name. OMIT unless the user names a city; omitted means the hub's configured default location.
    pub location: Option<String>,
    pub focus: Option<String>,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![definition::<Get>(
        "hub_weather_get",
        "Get weather. With no location argument it returns the hub's configured default location; only pass location when the user names another city",
    )]
}
