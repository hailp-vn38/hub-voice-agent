use crate::{
    CalendarDelta, CalendarEvent, CalendarSettings, CalendarSource, NewCalendarEvent, Store,
    runtime::effects::Effect,
};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

pub struct Feature {
    events: Vec<CalendarEvent>,
    last_sync: Option<i64>,
    online: Option<bool>,
    token: Option<String>,
    next_sync: i64,
    last_full_day: Option<i64>,
    created: Vec<String>,
}
impl Feature {
    pub fn load(store: &Store) -> anyhow::Result<Self> {
        let (events, token) = store.calendar_load()?;
        Ok(Self {
            events,
            last_sync: store.calendar_last_sync()?,
            online: None,
            token,
            next_sync: 0,
            last_full_day: None,
            created: Vec::new(),
        })
    }
    pub fn reset(&mut self) -> Vec<Effect> {
        self.events.clear();
        self.last_sync = None;
        self.online = None;
        self.created.clear();
        self.token = None;
        self.next_sync = 0;
        self.last_full_day = None;
        {
            let mut effects = vec![Effect::CancelJob("calendar-sync".into())];
            effects.extend(self.changed());
            effects
        }
    }
    pub fn snapshot(&self) -> Vec<Effect> {
        let events = self.events.clone();
        let last_sync = self.last_sync;
        let online = self.online;
        vec![Effect::Ui(Box::new(move |ui| {
            if ui.calendar == events
                && ui.calendar_last_sync == last_sync
                && ui.calendar_online == online
            {
                false
            } else {
                ui.calendar = events;
                ui.calendar_last_sync = last_sync;
                ui.calendar_online = online;
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
        let full = self.last_full_day != Some(now.div_euclid(86400));
        self.sync(source, settings, full)
    }
    fn sync(
        &self,
        source: Arc<dyn CalendarSource>,
        settings: CalendarSettings,
        full: bool,
    ) -> Vec<Effect> {
        let token = if full { None } else { self.token.clone() };
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
        if call.name == "hub_calendar_get" {
            let args: super::tools::List = match serde_json::from_value(call.arguments.clone()) {
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
            let mut effects = view.open(
                crate::Screen::Calendar,
                args.focus.map(crate::Focus::Calendar),
                now,
            );
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
        let args: super::tools::Create = match serde_json::from_value(call.arguments.clone()) {
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
        let dates = (
            args.start.parse::<jiff::civil::Date>(),
            args.end.parse::<jiff::civil::Date>(),
        );
        let valid = matches!((start, end), (Ok(a), Ok(b)) if a < b)
            || matches!(dates, (Ok(a), Ok(b)) if a < b);
        if args.summary.trim().is_empty() || !valid {
            return vec![Effect::Reply(
                call,
                Err(crate::features::timers::tools::error(
                    "Invalid event summary or time range",
                )),
            )];
        }
        let event = NewCalendarEvent {
            location: args.location,
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
    #[allow(clippy::too_many_arguments)]
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
            if let Ok(value) = result
                && let Ok(delta) = value.downcast::<(CalendarDelta, bool)>()
            {
                let (delta, full) = *delta;
                if full {
                    self.last_full_day = Some(now.div_euclid(86400));
                    self.events.retain(|event| self.created.contains(&event.id));
                }
                self.events
                    .retain(|event| !delta.removed.contains(&event.id));
                for event in delta.events {
                    self.created.retain(|id| id != &event.id);
                    self.upsert(event);
                }
                self.token = delta.sync_token;
                self.last_sync = Some(now);
                self.online = Some(true);
                self.prune(now);
                return self.changed();
            }
            self.online = Some(false);
            return self.snapshot();
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
                effects.extend(self.sync(source, settings, false));
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
            self.last_sync,
        ));
        effects
    }
}
