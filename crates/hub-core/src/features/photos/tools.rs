use schemars::{JsonSchema, schema_for};
use serde::Deserialize;
use voice_client::ToolDef;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImageInput {
    pub url: String,
    pub caption: String,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Show {
    pub images: Vec<ImageInput>,
}

pub fn definitions() -> Vec<ToolDef> {
    vec![ToolDef {
        name: "hub_show_image".into(),
        description: "Show one to six images from URLs".into(),
        input_schema: serde_json::to_value(schema_for!(Show)).unwrap(),
    }]
}
