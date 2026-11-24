use super::{effects::Effect, hub::Hub};
use crate::{
    Screen, Store,
    features::{Cx, Feature, registry, tool_args},
    runtime::jobs::JobResult,
};
use jiff::Zoned;
use voice_client::ToolCall;

impl Hub {
    /// Opens the tool's screen, then lets its feature handle the call.
    pub(super) fn on_tool(&mut self, call: ToolCall, store: &Store) -> Vec<Effect> {
        let now = self.now();
        let seconds = now.timestamp().as_second();
        let Some(screen) = registry::screen_for_hub_tool(&call.name) else {
            return vec![Effect::Reply(call, Err(tool_args::error("Unknown tool")))];
        };
        let mut effects = self.view.show(screen, seconds);
        // A routed screen always has a feature; a dropped call gets an internal error reply.
        effects.extend(
            self.with_feature(screen, &now, store, |feature, cx| {
                feature.handle_tool(cx, call)
            })
            .unwrap_or_default(),
        );
        effects
    }

    /// Hands a finished job to the feature that owns its id.
    pub(super) fn route_job(
        &mut self,
        screen: Screen,
        id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
        now: &Zoned,
        store: &Store,
    ) -> Vec<Effect> {
        self.with_feature(screen, now, store, |feature, cx| {
            feature.handle_job(cx, id, result, reply)
        })
        .unwrap_or_default()
    }
}

impl Hub {
    /// Runs `f` with the feature behind `screen` and the context it may touch.
    /// `None` for screens that are not features. The match has no wildcard, so a
    /// new `Screen` variant must be classified here before the crate compiles.
    fn with_feature<R>(
        &mut self,
        screen: Screen,
        now: &Zoned,
        store: &Store,
        f: impl FnOnce(&mut dyn Feature, &mut Cx) -> R,
    ) -> Option<R> {
        let Self {
            timers,
            countdowns,
            reminders,
            calendar,
            weather,
            home,
            photos,
            view,
            alerts,
            ..
        } = self;
        let feature: &mut dyn Feature = match screen {
            Screen::Timers => timers,
            Screen::Countdowns => countdowns,
            Screen::Reminders => reminders,
            Screen::Calendar => calendar,
            Screen::Weather => weather,
            Screen::Home => home,
            Screen::Photos => photos,
            Screen::Center | Screen::Menu | Screen::Ambient | Screen::Settings => return None,
        };
        let mut cx = Cx {
            view,
            alerts,
            store,
            now,
        };
        Some(f(feature, &mut cx))
    }
}
