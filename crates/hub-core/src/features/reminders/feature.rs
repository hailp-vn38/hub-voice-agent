use super::rules::{Reminder, ReminderDue, ReminderRepeat, parse_at};
use super::tools::{self, Tool};
use crate::{
    Alert, Store,
    alerts::Alerts,
    features::tool_args,
    runtime::effects::Effect,
    selector::{self, Selection},
};
use jiff::Zoned;
use voice_client::ToolCall;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReminderHistory {
    pub reminder: Reminder,
    pub missed: bool,
    pub at_seconds: i64,
}
pub struct Feature {
    items: Vec<Reminder>,
    history: Vec<ReminderHistory>,
}
impl Feature {
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        Ok(Self {
            items: store.reminders_load()?,
            history: store.reminder_history_load()?,
        })
    }
    pub fn snapshot(&self) -> Vec<Effect> {
        let items = self.items.clone();
        let history = self.history.clone();
        vec![Effect::Ui(Box::new(move |ui| {
            if ui.reminders == items && ui.reminder_history == history {
                return false;
            }
            ui.reminders = items;
            ui.reminder_history = history;
            true
        }))]
    }
    pub fn due(&self, now: &Zoned) -> Vec<Reminder> {
        self.items
            .iter()
            .filter(|item| item.due(now.clone()) == Some(ReminderDue::Alert))
            .cloned()
            .collect()
    }
    pub fn mark_missed(&mut self, now: &Zoned) -> Vec<Effect> {
        let ids: Vec<_> = self
            .items
            .iter()
            .filter(|item| item.due(now.clone()) == Some(ReminderDue::Missed))
            .map(|item| item.id)
            .collect();
        ids.into_iter()
            .flat_map(|id| self.finish(id, now, true))
            .collect()
    }
    /// Records an occurrence as reminded or missed and moves a repeating reminder
    /// to its next occurrence that is not already missed.
    pub fn finish(&mut self, id: i64, now: &Zoned, missed: bool) -> Vec<Effect> {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            let reminder = self.items[index].clone();
            self.history.insert(
                0,
                ReminderHistory {
                    at_seconds: now.timestamp().as_second(),
                    reminder,
                    missed,
                },
            );
            self.history.truncate(5);
            if let Some(next) = self.items[index].next_current(now) {
                self.items[index].at = next;
                self.items[index].scheduled_at = None;
            } else {
                self.items.remove(index);
            }
        }
        self.changed()
    }
    pub fn on_tool(&mut self, call: ToolCall, now: &Zoned, alerts: &mut Alerts) -> Vec<Effect> {
        let tool = match tools::parse(&call.name, call.arguments.clone()) {
            Ok(tool) => tool,
            Err(err) => return vec![Effect::Reply(call, Err(err))],
        };
        let result = match tool {
            Tool::Create(args) => self.create(args, now).map(|()| Vec::new()),
            Tool::List => return vec![self.reply(call)],
            Tool::Cancel(args) => self.cancel(&args.label, alerts),
            Tool::Snooze(args) => self.snooze(args.minutes, now, alerts),
        };
        match result {
            Ok(mut effects) => {
                effects.extend(self.changed());
                effects.push(self.reply(call));
                effects
            }
            Err(message) => vec![Effect::Reply(call, Err(tool_args::error(&message)))],
        }
    }
    fn create(&mut self, args: tools::Create, now: &Zoned) -> Result<(), String> {
        let at = parse_at(&args.at, now.time_zone());
        if args.label.trim().is_empty()
            || at.is_none()
            || args.weekdays.iter().any(|d| !(1..=7).contains(d))
            || (args.repeat == ReminderRepeat::Weekly && args.weekdays.is_empty())
        {
            return Err("Invalid reminder".into());
        }
        self.items.push(Reminder {
            id: self.items.iter().map(|r| r.id).max().unwrap_or(0) + 1,
            label: args.label,
            at: at.expect("checked above").to_string(),
            repeat: args.repeat,
            weekdays: args.weekdays,
            scheduled_at: None,
        });
        Ok(())
    }
    fn cancel(&mut self, label: &str, alerts: &mut Alerts) -> Result<Vec<Effect>, String> {
        let labels: Vec<_> = self.items.iter().map(|r| r.label.as_str()).collect();
        let index = match selector::select(&labels, label) {
            Selection::Found(index) => index,
            Selection::NotFound => return Err(format!("Reminder not found: {label}")),
            Selection::Ambiguous(found) => {
                return Err(format!("Ambiguous reminder; choices: {}", found.join(", ")));
            }
        };
        let removed = self.items.remove(index);
        Ok(match alerts.active() {
            Some(Alert::Reminder(active)) if active.id == removed.id => alerts.dismiss().1,
            _ => Vec::new(),
        })
    }
    fn snooze(
        &mut self,
        minutes: u64,
        now: &Zoned,
        alerts: &mut Alerts,
    ) -> Result<Vec<Effect>, String> {
        let Some(Alert::Reminder(active)) = alerts.active() else {
            return Err("No active reminder".into());
        };
        let id = active.id;
        let at = i64::try_from(minutes)
            .ok()
            .filter(|m| *m > 0)
            .and_then(|m| jiff::Span::new().try_minutes(m).ok())
            .and_then(|span| now.checked_add(span).ok())
            .ok_or("Invalid snooze duration")?;
        let item = self
            .items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or("Reminder not found")?;
        if item.scheduled_at.is_none() {
            item.scheduled_at = Some(item.at.clone());
        }
        item.at = at.to_string();
        Ok(alerts.dismiss().1)
    }
    fn reply(&self, call: ToolCall) -> Effect {
        Effect::Reply(call, Ok(serde_json::json!({"reminders": self.items})))
    }
    fn changed(&self) -> Vec<Effect> {
        let items = self.items.clone();
        let history = self.history.clone();
        let mut effects = vec![Effect::store(move |store| {
            store.reminders_save(&items, &history)
        })];
        effects.extend(self.snapshot());
        effects
    }
}
