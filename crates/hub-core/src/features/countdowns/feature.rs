use super::rules::Repeat;
use super::rules::{Countdown, CountdownView};
use crate::{
    Store,
    runtime::effects::Effect,
    selector::{Selection, select},
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
        views.sort_by_key(|item| item.days);
        views
    }
    pub fn snapshot(&self, today: Date) -> Vec<Effect> {
        let countdowns = self.views(today);
        vec![Effect::Ui(Box::new(move |state| {
            if state.countdowns == countdowns {
                return false;
            }
            state.countdowns = countdowns;
            true
        }))]
    }
    pub fn on_tool(
        &mut self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: jiff::Zoned,
    ) -> Vec<Effect> {
        let today = now.date();
        let request = match super::tools::parse(&call.name, call.arguments.clone()) {
            Ok(v) => v,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        let is_create = matches!(request, super::tools::Request::Create(_));
        let result: Result<bool, String> = match request {
            super::tools::Request::Create(args) => {
                if self.items.len() >= 8 {
                    Err("Maximum of 8 countdowns".into())
                } else if args.label.trim().is_empty() {
                    Err("Label required".into())
                } else {
                    let repeat = match args.repeat.as_str() {
                        "none" => Some(Repeat::None),
                        "yearly" => Some(Repeat::Yearly),
                        "yearly_lunar" => Some(Repeat::YearlyLunar),
                        _ => None,
                    };
                    if let Some(repeat) = repeat {
                        let id = self.items.iter().map(|item| item.id).max().unwrap_or(0) + 1;
                        let item = Countdown {
                            id,
                            label: args.label,
                            date: args.date,
                            repeat,
                            lunar_leap: args.lunar_leap,
                        };
                        let valid = if item.repeat == Repeat::YearlyLunar {
                            let parts: Vec<_> = item.date.split('-').collect();
                            if parts.len() != 3 {
                                false
                            } else {
                                match (parts[0].parse(), parts[1].parse(), parts[2].parse()) {
                                    (Ok(year), Ok(month), Ok(day)) => {
                                        crate::lunar::to_solar(crate::lunar::LunarDate {
                                            year,
                                            month,
                                            day,
                                            leap: item.lunar_leap,
                                        })
                                        .is_some()
                                    }
                                    _ => false,
                                }
                            }
                        } else {
                            item.date.parse::<Date>().is_ok() && !item.lunar_leap
                        };
                        if valid {
                            self.items.push(item);
                            Ok(true)
                        } else {
                            Err("Invalid date for calendar".into())
                        }
                    } else {
                        Err("Invalid repeat".into())
                    }
                }
            }
            super::tools::Request::Remove(args) => {
                let labels: Vec<_> = self.items.iter().map(|item| item.label.as_str()).collect();
                match select(&labels, &args.label) {
                    Selection::NotFound => Err(format!("Countdown not found: {}", args.label)),
                    Selection::Ambiguous(items) => Err(format!(
                        "Ambiguous countdown label: {}; choices: {}",
                        args.label,
                        items.join(", ")
                    )),
                    Selection::Found(item) => {
                        let index = labels.iter().position(|label| *label == item).unwrap();
                        self.items.remove(index);
                        Ok(true)
                    }
                }
            }
            super::tools::Request::List => Ok(false),
        };
        match result {
            Err(message) => vec![Effect::Reply(call, Err(super::tools::error(&message)))],
            Ok(changed) => {
                let mut effects = Vec::new();
                if changed {
                    effects.push(Effect::PersistCountdowns(self.items.clone()));
                    effects.extend(self.snapshot(today));
                }
                let focus = if is_create {
                    self.items
                        .last()
                        .map(|item| crate::Focus::Countdown(item.id))
                } else {
                    None
                };
                effects.extend(view.open(
                    crate::Screen::Countdowns,
                    focus,
                    now.timestamp().as_second(),
                ));
                effects.push(Effect::Reply(
                    call,
                    Ok(serde_json::json!({"countdowns": self.views(today)})),
                ));
                effects
            }
        }
    }
}
