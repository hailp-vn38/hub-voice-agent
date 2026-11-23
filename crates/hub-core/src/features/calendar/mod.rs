mod store;
pub(crate) mod tools;
use crate::{
    CalendarDelta, CalendarEvent, CalendarSettings, CalendarSource, NewCalendarEvent, Store,
    runtime::effects::Effect,
};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

pub struct Feature {
    events: Vec<CalendarEvent>,
    token: Option<String>,
    next_sync: i64,
    created: Vec<String>,
}
impl Feature {
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        let (events, token) = store.calendar_load()?;
        Ok(Self {
            events,
            token,
            next_sync: 0,
            created: Vec::new(),
        })
    }
    pub fn reset(&mut self) -> Vec<Effect> {
        self.events.clear();
        self.created.clear();
        self.token = None;
        self.next_sync = 0;
        self.changed()
    }
    pub fn snapshot(&self) -> Vec<Effect> {
        let events = self.events.clone();
        vec![Effect::Ui(Box::new(move |ui| {
            if ui.calendar == events {
                false
            } else {
                ui.calendar = events;
                true
            }
        }))]
    }
    pub fn tick(
        &mut self,
        source: Arc<dyn CalendarSource>,
        settings: CalendarSettings,
        now: i64,
    ) -> Vec<Effect> {
        if now < self.next_sync || settings.refresh_token.is_empty() {
            return vec![];
        }
        self.next_sync = now.saturating_add(300);
        self.sync(source, settings)
    }
    fn sync(&self, source: Arc<dyn CalendarSource>, settings: CalendarSettings) -> Vec<Effect> {
        let token = self.token.clone();
        vec![Effect::Spawn {
            job: "calendar-sync".into(),
            work: Box::pin(async move {
                let mut delta = source.list(settings.clone(), token.clone()).await?;
                let full = token.is_none() || delta.expired;
                if delta.expired {
                    delta = source.list(settings, None).await?;
                    if delta.expired {
                        anyhow::bail!("Calendar sync token expired during full sync");
                    }
                }
                Ok(Box::new((delta, full)) as _)
            }),
            reply: None,
            timeout: Duration::from_secs(30),
        }]
    }
    pub fn on_tool(
        &mut self,
        view: &mut crate::view::View,
        call: ToolCall,
        source: Arc<dyn CalendarSource>,
        settings: CalendarSettings,
        now: i64,
    ) -> Vec<Effect> {
        if call.name == "hub_calendar_list" {
            let args: tools::List = match serde_json::from_value(call.arguments.clone()) {
                Ok(v) => v,
                Err(e) => {
                    return vec![Effect::Reply(
                        call,
                        Err(crate::features::timers::tools::error(&e.to_string())),
                    )];
                }
            };
            let events: Vec<_> = self
                .events
                .iter()
                .filter(|event| {
                    args.start.as_ref().is_none_or(|s| event.end > *s)
                        && args.end.as_ref().is_none_or(|e| event.start < *e)
                })
                .cloned()
                .collect();
            let mut effects = view.open(crate::Screen::Calendar, None, now);
            effects.push(Effect::Reply(
                call,
                Ok(serde_json::json!({"events": events})),
            ));
            return effects;
        }
        if call.name != "hub_calendar_create" {
            return vec![Effect::Reply(
                call,
                Err(crate::features::timers::tools::error(
                    "Unknown calendar tool",
                )),
            )];
        }
        let args: tools::Create = match serde_json::from_value(call.arguments.clone()) {
            Ok(v) => v,
            Err(e) => {
                return vec![Effect::Reply(
                    call,
                    Err(crate::features::timers::tools::error(&e.to_string())),
                )];
            }
        };
        let start = args.start.parse::<jiff::Timestamp>();
        let end = args.end.parse::<jiff::Timestamp>();
        if args.summary.trim().is_empty() || !matches!((start, end), (Ok(a), Ok(b)) if a < b) {
            return vec![Effect::Reply(
                call,
                Err(crate::features::timers::tools::error(
                    "Invalid event summary or time range",
                )),
            )];
        }
        let event = NewCalendarEvent {
            summary: args.summary,
            start: args.start,
            end: args.end,
        };
        let id = format!("calendar-create-{}", uuid::Uuid::new_v4());
        vec![Effect::Spawn {
            job: id,
            work: Box::pin(async move { Ok(Box::new(source.create(settings, event).await?) as _) }),
            reply: Some(call),
            timeout: Duration::from_secs(30),
        }]
    }
    pub fn on_job(
        &mut self,
        id: &str,
        result: anyhow::Result<Box<dyn std::any::Any + Send>>,
        reply: Option<ToolCall>,
        view: &mut crate::view::View,
        source: Arc<dyn CalendarSource>,
        settings: CalendarSettings,
        now: i64,
    ) -> Vec<Effect> {
        if id == "calendar-sync" {
            if let Ok(value) = result {
                if let Ok(delta) = value.downcast::<(CalendarDelta, bool)>() {
                    let (delta, full) = *delta;
                    if full {
                        self.events.retain(|event| self.created.contains(&event.id));
                    }
                    self.events
                        .retain(|event| !delta.removed.contains(&event.id));
                    for event in delta.events {
                        self.created.retain(|id| id != &event.id);
                        self.upsert(event);
                    }
                    self.token = delta.sync_token;
                    self.prune(now);
                    return self.changed();
                }
            }
            return vec![];
        }
        let Some(call) = reply else {
            return vec![];
        };
        match result.and_then(|value| {
            value
                .downcast::<CalendarEvent>()
                .map(|v| *v)
                .map_err(|_| anyhow::anyhow!("Invalid calendar result"))
        }) {
            Ok(event) => {
                self.created.push(event.id.clone());
                self.upsert(event.clone());
                let mut effects = self.changed();
                effects.extend(view.open(
                    crate::Screen::Calendar,
                    Some(crate::Focus::Calendar(event.id.clone())),
                    now,
                ));
                effects.push(Effect::Reply(call, Ok(serde_json::json!({"event": event}))));
                effects.extend(self.sync(source, settings));
                effects
            }
            Err(_) => vec![Effect::ReplyError(call, "Calendar request failed")],
        }
    }
    fn prune(&mut self, now: i64) {
        let Ok(timestamp) = jiff::Timestamp::from_second(now) else {
            return;
        };
        let date = timestamp.to_zoned(jiff::tz::TimeZone::UTC).date();
        let (Ok(from), Ok(to)) = (
            date.checked_sub(jiff::Span::new().days(30)),
            date.checked_add(jiff::Span::new().days(60)),
        ) else {
            return;
        };
        let from = from.to_string();
        let to = to.to_string();
        self.events.retain(|event| {
            self.created.contains(&event.id)
                || (event.end.get(..10).is_some_and(|end| end >= from.as_str())
                    && event
                        .start
                        .get(..10)
                        .is_some_and(|start| start <= to.as_str()))
        });
    }
    fn upsert(&mut self, event: CalendarEvent) {
        self.events.retain(|old| old.id != event.id);
        self.events.push(event);
        self.events.sort_by(|a, b| a.start.cmp(&b.start));
    }
    fn changed(&self) -> Vec<Effect> {
        let mut effects = self.snapshot();
        effects.push(Effect::PersistCalendar(
            self.events.clone(),
            self.token.clone(),
        ));
        effects
    }
}
