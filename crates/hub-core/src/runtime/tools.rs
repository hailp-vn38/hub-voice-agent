use super::{effects::Effect, hub::Hub};
use crate::{Screen, Store, features::tool_args, view::screen_for_hub_tool};
use voice_client::ToolCall;

impl Hub {
    /// Opens the tool's screen, then lets its feature handle the call.
    pub(super) fn on_tool(&mut self, call: ToolCall, store: &Store) -> Vec<Effect> {
        let now = self.now();
        let seconds = now.timestamp().as_second();
        let Some(screen) = screen_for_hub_tool(&call.name) else {
            return vec![Effect::Reply(call, Err(tool_args::error("Unknown tool")))];
        };
        let mut effects = self.view.show(screen, seconds);
        let view = &mut self.view;
        effects.extend(match screen {
            Screen::Calendar => self.calendar.on_tool(view, call, &now),
            Screen::Timers => self.timers.on_tool(view, call, seconds),
            Screen::Countdowns => self.countdowns.on_tool(view, call, &now),
            Screen::Reminders => self.reminders.on_tool(call, &now, &mut self.alerts),
            Screen::Weather => self.weather.on_tool(view, call, seconds, store),
            Screen::Home => self.home.on_tool(call),
            Screen::Photos => self.photos.on_tool(view, call, seconds, store),
            _ => vec![Effect::Reply(call, Err(tool_args::error("Unknown tool")))],
        });
        effects
    }
}
