mod rules;
mod store;
pub(crate) mod tools;

use crate::{Store, runtime::effects::Effect};
pub use rules::Timer;
use rules::Timers;
use tools::TimerTool;
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
    pub fn on_tool(
        &mut self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: i64,
    ) -> Vec<Effect> {
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
            TimerTool::List => {
                let mut effects = view.open(crate::Screen::Timers, None, now);
                effects.push(Effect::Reply(
                    call,
                    Ok(serde_json::json!({"timers": self.timers.items})),
                ));
                return effects;
            }
        };
        match result {
            Ok(()) => {
                let mut effects = vec![Effect::Persist(self.timers.items.clone())];
                effects.extend(self.snapshot());
                let focus = if is_start {
                    self.timers
                        .items
                        .last()
                        .map(|timer| crate::Focus::Timer(timer.id))
                } else {
                    None
                };
                effects.extend(view.open(crate::Screen::Timers, focus, now));
                effects.push(Effect::Reply(
                    call,
                    Ok(serde_json::json!({"timers": self.timers.items})),
                ));
                effects
            }
            Err(err) => vec![Effect::Reply(call, Err(tools::error(&err)))],
        }
    }
    pub fn tick(&mut self, now: i64) -> Vec<Effect> {
        let _due = self.timers.due(now);
        Vec::new()
    }
}
