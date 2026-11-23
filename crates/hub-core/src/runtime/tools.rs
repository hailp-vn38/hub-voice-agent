use super::effects::Effect;
use crate::features::timers::{Feature, tools};
use voice_client::ToolCall;

pub(super) fn route(
    timers: &mut Feature,
    countdowns: &mut crate::features::countdowns::Feature,
    call: ToolCall,
    now: jiff::Zoned,
) -> Vec<Effect> {
    if call.name.starts_with("hub_timer_") {
        timers.on_tool(call, now.timestamp().as_second())
    } else if call.name.starts_with("hub_countdown_") {
        countdowns.on_tool(call, now.date())
    } else {
        vec![Effect::Reply(call, Err(tools::error("Unknown tool")))]
    }
}
