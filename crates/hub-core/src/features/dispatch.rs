//! The narrow `Feature` interface the runtime calls, and each feature's adapter
//! onto its own richer methods.

use super::{calendar, countdowns, home, photos, reminders, timers, weather};
use crate::{Store, alerts::Alerts, runtime::effects::Effect, runtime::jobs::JobResult, view::View};
use jiff::Zoned;
use voice_client::ToolCall;

/// What a feature may touch while handling one call; built per call by the runtime.
pub(crate) struct Cx<'a> {
    pub view: &'a mut View,
    pub alerts: &'a mut Alerts,
    pub store: &'a Store,
    pub now: &'a Zoned,
}

impl Cx<'_> {
    fn seconds(&self) -> i64 {
        self.now.timestamp().as_second()
    }
}

pub(crate) trait Feature {
    /// A Device MCP tool call routed here by `registry::screen_for_hub_tool`.
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect>;

    /// A finished or timed-out job routed here by `registry::screen_for_job`.
    fn handle_job(
        &mut self,
        _cx: &mut Cx,
        _id: &str,
        _result: JobResult,
        _reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        Vec::new()
    }
}

impl Feature for timers::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        let seconds = cx.seconds();
        self.on_tool(cx.view, call, seconds)
    }
}

impl Feature for countdowns::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        self.on_tool(cx.view, call, cx.now)
    }
}

impl Feature for reminders::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        self.on_tool(call, cx.now, cx.alerts)
    }
}

impl Feature for calendar::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        self.on_tool(cx.view, call, cx.now)
    }

    fn handle_job(
        &mut self,
        cx: &mut Cx,
        id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        self.on_job(id, result, reply, cx.view, cx.now)
    }
}

impl Feature for weather::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        let seconds = cx.seconds();
        self.on_tool(cx.view, call, seconds, cx.store)
    }

    fn handle_job(
        &mut self,
        cx: &mut Cx,
        _id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        let seconds = cx.seconds();
        self.on_job(result, reply, seconds)
    }
}

impl Feature for home::Feature {
    fn handle_tool(&mut self, _cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        self.on_tool(call)
    }

    fn handle_job(
        &mut self,
        _cx: &mut Cx,
        id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        self.on_job(id, result, reply)
    }
}

impl Feature for photos::Feature {
    fn handle_tool(&mut self, cx: &mut Cx, call: ToolCall) -> Vec<Effect> {
        let seconds = cx.seconds();
        self.on_tool(cx.view, call, seconds, cx.store)
    }

    fn handle_job(
        &mut self,
        cx: &mut Cx,
        _id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        let seconds = cx.seconds();
        self.on_job(result, reply, seconds, cx.view)
    }
}
