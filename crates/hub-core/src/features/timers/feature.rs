use super::rules::{Timer, Timers};
use super::tools::{self, TimerTool};
use crate::{Focus, Screen, Store, features::tool_args, runtime::effects::Effect, view::View};
use voice_client::ToolCall;

pub struct Feature {
    timers: Timers,
}
impl Feature {
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        Ok(Self {
            timers: Timers {
                items: store.timers_load()?,
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
