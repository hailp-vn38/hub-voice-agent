use crate::features::tool_args::definition;
use schemars::JsonSchema;
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Args {
    pub entity: String,
}
pub fn definitions() -> Vec<ToolDef> {
    vec![
        definition::<Args>("hub_home_get", "Get a device state"),
        definition::<Args>("hub_home_turn_on", "Turn on a device"),
        definition::<Args>("hub_home_turn_off", "Turn off a device"),
    ]
}
