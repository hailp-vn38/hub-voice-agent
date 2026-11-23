mod rules;
pub(crate) mod speech;
mod store;
pub(crate) mod tools;

use crate::{Store, runtime::effects::Effect};
pub use rules::{Due, Reminder, Repeat};
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
    pub fn due(&self, now: jiff::Zoned) -> Vec<Reminder> {
        self.items
            .iter()
            .filter(|item| item.due(now.clone()) == Some(Due::Alert))
            .cloned()
            .collect()
    }
    pub fn mark_missed(&mut self, now: jiff::Zoned) -> Vec<Effect> {
        let ids: Vec<_> = self
            .items
            .iter()
            .filter(|item| item.due(now.clone()) == Some(Due::Missed))
            .map(|item| item.id)
            .collect();
        let mut effects = Vec::new();
        for id in ids {
            effects.extend(self.finish(id, now.timestamp().as_second(), true));
        }
        effects
    }
    pub fn finish(&mut self, id: i64, now: i64, missed: bool) -> Vec<Effect> {
        if let Some(index) = self.items.iter().position(|item| item.id == id) {
            let reminder = self.items[index].clone();
            self.history.insert(
                0,
                ReminderHistory {
                    reminder,
                    missed,
                    at_seconds: now,
                },
            );
            self.history.truncate(5);
            if let Some(next) = self.items[index].next_occurrence() {
                self.items[index].at = next;
                self.items[index].scheduled_at = None;
            } else {
                self.items.remove(index);
            }
        }
        let mut effects = vec![Effect::PersistReminders(
            self.items.clone(),
            self.history.clone(),
        )];
        effects.extend(self.snapshot());
        effects
    }
    pub fn on_tool(
        &mut self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: i64,
        alerts: &mut crate::alerts::Alerts,
    ) -> Vec<Effect> {
        let tool = match tools::parse(&call.name, call.arguments.clone()) {
            Ok(tool) => tool,
            Err(err) => return vec![Effect::Reply(call, Err(err))],
        };
        match tool {
            tools::Tool::Create(args) => {
                let repeat = match args.repeat.as_str() {
                    "none" => Repeat::None,
                    "daily" => Repeat::Daily,
                    "weekly" => Repeat::Weekly,
                    _ => {
                        return vec![Effect::Reply(
                            call,
                            Err(crate::features::timers::tools::error("Invalid repeat")),
                        )];
                    }
                };
                if args.label.trim().is_empty()
                    || args.at.parse::<jiff::Zoned>().is_err()
                    || args.weekdays.iter().any(|d| !(1..=7).contains(d))
                    || (repeat == Repeat::Weekly && args.weekdays.is_empty())
                {
                    return vec![Effect::Reply(
                        call,
                        Err(crate::features::timers::tools::error("Invalid reminder")),
                    )];
                }
                let id = self.items.iter().map(|r| r.id).max().unwrap_or(0) + 1;
                self.items.push(Reminder {
                    id,
                    label: args.label,
                    at: args.at,
                    repeat,
                    weekdays: args.weekdays,
                    scheduled_at: None,
                });
            }
            tools::Tool::List => {}
            tools::Tool::Cancel(args) => {
                let labels: Vec<_> = self.items.iter().map(|r| r.label.as_str()).collect();
                let index = match crate::selector::select(&labels, &args.label) {
                    crate::selector::Selection::NotFound => {
                        return vec![Effect::Reply(
                            call,
                            Err(crate::features::timers::tools::error("Reminder not found")),
                        )];
                    }
                    crate::selector::Selection::Ambiguous(found) => {
                        return vec![Effect::Reply(
                            call,
                            Err(crate::features::timers::tools::error(&format!(
                                "Ambiguous reminder; choices: {}",
                                found.join(", ")
                            ))),
                        )];
                    }
                    crate::selector::Selection::Found(found) => {
                        labels.iter().position(|label| *label == found).unwrap()
                    }
                };
                self.items.remove(index);
            }
            tools::Tool::Snooze(args) => {
                let id = match alerts.active() {
                    Some(crate::Alert::Reminder(reminder)) => reminder.id,
                    _ => {
                        return vec![Effect::Reply(
                            call,
                            Err(crate::features::timers::tools::error("No active reminder")),
                        )];
                    }
                };
                let Some(seconds) = i64::try_from(args.minutes)
                    .ok()
                    .and_then(|m| m.checked_mul(60))
                    .filter(|s| *s > 0)
                    .and_then(|s| now.checked_add(s))
                else {
                    return vec![Effect::Reply(
                        call,
                        Err(crate::features::timers::tools::error(
                            "Invalid snooze duration",
                        )),
                    )];
                };
                let Some(item) = self.items.iter_mut().find(|item| item.id == id) else {
                    return vec![Effect::Reply(
                        call,
                        Err(crate::features::timers::tools::error("Reminder not found")),
                    )];
                };
                let at: jiff::Zoned = item.at.parse().expect("validated reminder time");
                let Some(timestamp) = jiff::Timestamp::from_second(seconds).ok() else {
                    return vec![Effect::Reply(
                        call,
                        Err(crate::features::timers::tools::error(
                            "Invalid snooze duration",
                        )),
                    )];
                };
                if item.scheduled_at.is_none() {
                    item.scheduled_at = Some(item.at.clone());
                }
                item.at = timestamp.to_zoned(at.time_zone().clone()).to_string();
                let (_, mut effects) = alerts.dismiss();
                effects.push(Effect::PersistReminders(
                    self.items.clone(),
                    self.history.clone(),
                ));
                effects.extend(self.snapshot());
                effects.push(Effect::Reply(
                    call,
                    Ok(serde_json::json!({"reminders": self.items})),
                ));
                return effects;
            }
        }
        let mut effects = vec![Effect::PersistReminders(
            self.items.clone(),
            self.history.clone(),
        )];
        effects.extend(self.snapshot());
        effects.extend(view.open(crate::Screen::Reminders, None, now));
        effects.push(Effect::Reply(
            call,
            Ok(serde_json::json!({"reminders": self.items})),
        ));
        effects
    }
}
