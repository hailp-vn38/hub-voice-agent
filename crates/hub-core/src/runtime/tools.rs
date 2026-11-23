use super::effects::Effect;
use crate::features::timers::{Feature, tools};
use voice_client::ToolCall;

pub(super) fn route(timers: &mut Feature, call: ToolCall, now: i64) -> Vec<Effect> {
    if call.name.starts_with("hub_timer_") {
        timers.on_tool(call, now)
    } else {
        vec![Effect::Reply(call, Err(tools::error("Unknown tool")))]
    }
}
