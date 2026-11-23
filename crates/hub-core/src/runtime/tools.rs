use super::effects::Effect;
use crate::features::timers::{Feature, tools};
use voice_client::ToolCall;

pub(super) fn route(
    timers: &mut Feature,
    reminders: &mut crate::features::reminders::Feature,
    countdowns: &mut crate::features::countdowns::Feature,
    view: &mut crate::view::View,
    alerts: &mut crate::alerts::Alerts,
    call: ToolCall,
    now: jiff::Zoned,
) -> Vec<Effect> {
    if call.name.starts_with("hub_reminder_") {
        reminders.on_tool(view, call, now.timestamp().as_second(), alerts)
    } else if call.name.starts_with("hub_timer_") {
        timers.on_tool(view, call, now.timestamp().as_second())
    } else if call.name.starts_with("hub_countdown_") {
        countdowns.on_tool(view, call, now)
    } else {
        vec![Effect::Reply(call, Err(tools::error("Unknown tool")))]
    }
}
