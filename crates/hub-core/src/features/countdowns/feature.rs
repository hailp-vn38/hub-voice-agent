use super::rules::{Countdown, CountdownView};
use super::tools::{self, Request};
use crate::{
    Focus, Screen, Store,
    features::tool_args,
    runtime::effects::Effect,
    selector::{self, Selection},
    view::View,
};
use jiff::civil::Date;
use voice_client::ToolCall;

pub struct Feature {
    items: Vec<Countdown>,
}
impl Feature {
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        Ok(Self {
            items: store.countdowns_load()?,
        })
    }
    fn views(&self, today: Date) -> Vec<CountdownView> {
        let mut views: Vec<_> = self
            .items
            .iter()
            .filter_map(|item| item.view(today))
            .collect();
        views.sort_by_key(|item| (item.days < 0, item.days.abs()));
        views
    }
    pub fn snapshot(&self, today: Date) -> Vec<Effect> {
        let countdowns = self.views(today);
        vec![Effect::Ui(Box::new(move |state| {
            let arrived_today = countdowns.iter().any(|countdown| {
                countdown.days == 0
                    && !state
                        .countdowns
                        .iter()
                        .any(|current| current.id == countdown.id && current.days == 0)
            });
            let changed = state.countdowns != countdowns;
            if !changed && !arrived_today {
                return false;
            }
            if changed {
                state.countdowns = countdowns;
            }
            if arrived_today {
                state.events.confetti = state.events.confetti.saturating_add(1);
            }
            true
        }))]
    }

    pub fn remove_id(&mut self, id: i64, today: Date) -> Vec<Effect> {
        let Some(index) = self.items.iter().position(|item| item.id == id) else {
            return Vec::new();
        };
        self.items.remove(index);
        let items = self.items.clone();
        let mut effects = vec![Effect::store(move |store| store.countdowns_save(&items))];
        effects.extend(self.snapshot(today));
        effects
    }

    pub fn on_tool(&mut self, view: &mut View, call: ToolCall, now: &jiff::Zoned) -> Vec<Effect> {
        let today = now.date();
        let request = match tools::parse(&call.name, call.arguments.clone()) {
            Ok(request) => request,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        let result = match request {
            Request::Create(args) => self.create(args, now).map(Some),
            Request::Remove(args) => self.remove(&args.label).map(|()| None),
            Request::List => {
                return vec![self.reply(call, today)];
            }
        };
        let created = match result {
            Ok(created) => created,
            Err(message) => return vec![Effect::Reply(call, Err(tool_args::error(&message)))],
        };
        let mut effects = self.changed(today);
        if let Some(id) = created {
            effects.extend(view.open(
                Screen::Countdowns,
                Some(Focus::Countdown(id)),
                now.timestamp().as_second(),
            ));
        }
        effects.push(self.reply(call, today));
        effects
    }
    fn create(&mut self, args: tools::Create, now: &jiff::Zoned) -> Result<i64, String> {
        if self.items.len() >= 8 {
            return Err("Maximum of 8 countdowns".into());
        }
        if args.label.trim().is_empty() {
            return Err("Label required".into());
        }
        let item = Countdown {
            id: self.items.iter().map(|item| item.id).max().unwrap_or(0) + 1,
            label: args.label,
            date: args.date,
            repeat: args.repeat,
            lunar_leap: args.lunar_leap,
            created_at: now.to_string(),
        };
        if !item.is_valid() {
            return Err("Invalid date for calendar".into());
        }
        let id = item.id;
        self.items.push(item);
        Ok(id)
    }
    fn remove(&mut self, label: &str) -> Result<(), String> {
        let labels: Vec<_> = self.items.iter().map(|item| item.label.as_str()).collect();
        match selector::select(&labels, label) {
            Selection::NotFound => Err(format!("Countdown not found: {label}")),
            Selection::Ambiguous(items) => Err(format!(
                "Ambiguous countdown label: {label}; choices: {}",
                items.join(", ")
            )),
            Selection::Found(index) => {
                self.items.remove(index);
                Ok(())
            }
        }
    }
    fn reply(&self, call: ToolCall, today: Date) -> Effect {
        Effect::Reply(
            call,
            Ok(serde_json::json!({"countdowns": self.views(today)})),
        )
    }

    fn changed(&self, today: Date) -> Vec<Effect> {
        let items = self.items.clone();
        let mut effects = vec![Effect::store(move |store| store.countdowns_save(&items))];
        effects.extend(self.snapshot(today));
        effects
    }
}
