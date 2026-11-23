use super::rules::{Timer, Timers};
use super::tools::{self, TimerTool};
use crate::{
    Focus, Screen, Store, TimerInput, features::tool_args, runtime::effects::Effect, view::View,
};
use voice_client::ToolCall;

pub struct Feature {
    timers: Timers,
}
impl Feature {
    pub fn load(store: &mut Store, now: i64) -> anyhow::Result<Self> {
        Ok(Self {
            timers: Timers {
                items: store.timers_load(now)?,
            },
        })
    }
    pub fn snapshot(&self) -> Vec<Effect> {
        let timers = self.timers.items.clone();
        vec![Effect::Ui(Box::new(move |state| {
            if state.timers == timers {
                return false;
            }
            state.timers = timers;
            true
        }))]
    }
    pub fn on_tool(&mut self, view: &mut View, call: ToolCall, now: i64) -> Vec<Effect> {
        let tool = match tools::parse(&call.name, call.arguments.clone()) {
            Ok(tool) => tool,
            Err(err) => return vec![Effect::Reply(call, Err(err))],
        };
        let is_start = matches!(tool, TimerTool::Start(_));
        let result = match tool {
            TimerTool::Start(args) => i64::try_from(args.minutes)
                .ok()
                .and_then(|m| m.checked_mul(60))
                .ok_or_else(|| "Duration too large".to_string())
                .and_then(|s| self.timers.start(args.label, s, now)),
            TimerTool::Adjust(args) => {
                self.timers
                    .adjust(&args.label, args.delta_seconds, args.paused, now)
            }
            TimerTool::Cancel(args) => self.timers.cancel(&args.label),
            TimerTool::List => return vec![self.reply(call)],
        };
        if let Err(err) = result {
            return vec![Effect::Reply(call, Err(tool_args::error(&err)))];
        }
        let mut effects = self.changed();
        if is_start && let Some(timer) = self.timers.items.last() {
            effects.extend(view.open(Screen::Timers, Some(Focus::Timer(timer.id)), now));
        }
        effects.push(self.reply(call));
        effects
    }
    pub fn due(&self, now: i64) -> Vec<Timer> {
        self.timers.due(now)
    }
    pub fn on_input(&mut self, view: &mut View, input: TimerInput, now: i64) -> Vec<Effect> {
        let result = match input {
            TimerInput::QuickStart(minutes) => self
                .timers
                .quick_start(minutes, now)
                .map(|id| Some(Focus::Timer(id))),
            TimerInput::TogglePause(id) => self
                .timers
                .toggle_pause(id, now)
                .map(|()| Some(Focus::Timer(id))),
            TimerInput::AddMinute(id) => self
                .timers
                .add_minute(id, now)
                .map(|()| Some(Focus::Timer(id))),
            TimerInput::Cancel(id) => self.timers.cancel_id(id).map(|()| match view.focus() {
                Some(Focus::Timer(focused)) if *focused == id => None,
                Some(focus) => Some(focus.clone()),
                None => None,
            }),
        };
        let Ok(focus) = result else {
            return Vec::new();
        };
        let mut effects = self.changed();
        effects.extend(view.open(Screen::Timers, focus, now));
        effects
    }
    pub fn dismiss(&mut self, id: i64) -> Vec<Effect> {
        self.timers.items.retain(|timer| timer.id != id);
        self.changed()
    }
    fn reply(&self, call: ToolCall) -> Effect {
        Effect::Reply(call, Ok(serde_json::json!({"timers": self.timers.items})))
    }
    fn changed(&self) -> Vec<Effect> {
        let items = self.timers.items.clone();
        let mut effects = vec![Effect::store(move |store| store.timers_save(&items))];
        effects.extend(self.snapshot());
        effects
    }
}
