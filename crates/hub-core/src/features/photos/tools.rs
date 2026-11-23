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

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct List {}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Control {
    pub action: Action,
    pub id: Option<String>,
}
#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Open,
    Next,
    Previous,
    Pin,
    Unpin,
}

pub fn definitions() -> Vec<ToolDef> {
    vec![
        ToolDef {
            name: "hub_show_image".into(),
            description: "Show one to six images from URLs".into(),
            input_schema: serde_json::to_value(schema_for!(Show)).unwrap(),
        },
        ToolDef {
            name: "hub_image_list".into(),
            description: "List cached images".into(),
            input_schema: serde_json::to_value(schema_for!(List)).unwrap(),
        },
        ToolDef {
            name: "hub_image_control".into(),
            description: "Open, navigate, pin or unpin an image".into(),
            input_schema: serde_json::to_value(schema_for!(Control)).unwrap(),
        },
    ]
}
