use super::time::{instant, valid_range, window};
use super::tools;
use crate::{
    CalendarDelta, CalendarEvent, CalendarQuery, CalendarSettings, CalendarSource, Focus,
    NewCalendarEvent, Screen, Store,
    features::tool_args,
    runtime::{effects::Effect, jobs::JobResult},
    ui_state::{CalendarPendingEvent, CalendarPendingState},
    view::View,
};
use jiff::{Timestamp, Zoned, civil::Date};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

pub(super) const SYNC_JOB: &str = "calendar-sync";
pub(super) const CREATE_JOB_PREFIX: &str = "calendar-create-";
const SYNC_SECONDS: i64 = 300;
const SAVED_SECONDS: i64 = 3;

pub struct Feature {
    source: Arc<dyn CalendarSource>,
    settings: CalendarSettings,
    events: Vec<CalendarEvent>,
    last_sync: Option<i64>,
    online: Option<bool>,
    token: Option<String>,
    next_sync: i64,
    last_full_day: Option<Date>,
    /// Events created here that a sync has not returned yet.
    created: Vec<String>,
    pending: Vec<CalendarPendingEvent>,
}
impl Feature {
    pub fn load(
        source: Arc<dyn CalendarSource>,
        settings: CalendarSettings,
        store: &Store,
    ) -> anyhow::Result<Self> {
        let (events, token) = store.calendar_load()?;
        Ok(Self {
            source,
            settings,
            events,
            last_sync: store.calendar_last_sync()?,
            online: None,
            token,
            next_sync: 0,
            last_full_day: None,
            created: Vec::new(),
            pending: Vec::new(),
        })
    }
    /// Drops the cache for a new account and syncs it at once.
    pub fn reset(&mut self, settings: CalendarSettings, now: &Zoned) -> Vec<Effect> {
        let source = self.source.clone();
        *self = Self {
            source,
            settings,
            events: Vec::new(),
            last_sync: None,
            online: None,
            token: None,
            next_sync: 0,
            last_full_day: None,
            created: Vec::new(),
            pending: Vec::new(),
        };
        let mut effects = vec![Effect::CancelJob(SYNC_JOB.into())];
        effects.extend(self.changed());
        effects.extend(self.tick(now));
        effects
    }
    pub fn snapshot(&self) -> Vec<Effect> {
        let events = self.events.clone();
        let last_sync = self.last_sync;
        let online = self.online;
        let pending = self.pending.clone();
        vec![Effect::Ui(Box::new(move |ui| {
            if ui.calendar == events
                && ui.calendar_last_sync == last_sync
                && ui.calendar_online == online
                && ui.calendar_pending == pending
            {
                return false;
            }
            ui.calendar = events;
            ui.calendar_last_sync = last_sync;
            ui.calendar_online = online;
            ui.calendar_pending = pending;
            true
        }))]
    }
    /// Syncs every 5 minutes; the first sync of each local day is a full one.
    pub fn tick(&mut self, now: &Zoned) -> Vec<Effect> {
        let seconds = now.timestamp().as_second();
        let previous = self.pending.len();
        self.pending.retain(|pending| {
            pending.state != CalendarPendingState::Saved
                || pending
                    .saved_at
                    .is_none_or(|saved| seconds.saturating_sub(saved) < SAVED_SECONDS)
        });
        let mut effects = if previous != self.pending.len() {
            self.snapshot()
        } else {
            Vec::new()
        };
        if seconds < self.next_sync || self.settings.refresh_token.is_empty() {
            return effects;
        }
        self.next_sync = seconds.saturating_add(SYNC_SECONDS);
        let full = self.last_full_day != Some(now.date());
        effects.extend(self.sync(full, now));
        effects
    }
    fn sync(&self, full: bool, now: &Zoned) -> Vec<Effect> {
        let Some((time_min, time_max)) = window(now) else {
            return vec![];
        };
        let full_query = CalendarQuery::Full { time_min, time_max };
        let query = match &self.token {
            Some(token) if !full => CalendarQuery::Since(token.clone()),
            _ => full_query.clone(),
        };
        let source = self.source.clone();
        let settings = self.settings.clone();
        vec![Effect::Spawn {
            job: SYNC_JOB.into(),
            work: Box::pin(async move {
                let mut full = matches!(query, CalendarQuery::Full { .. });
                let mut delta = source.list(settings.clone(), query).await?;
                if delta.expired {
                    full = true;
                    delta = source.list(settings, full_query).await?;
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
    pub fn on_tool(&mut self, view: &mut View, call: ToolCall, now: &Zoned) -> Vec<Effect> {
        match call.name.as_str() {
            "hub_calendar_get" => self.get(view, call, now),
            "hub_calendar_create" => self.create(view, call, now),
            _ => vec![Effect::Reply(
                call,
                Err(tool_args::error("Unknown calendar tool")),
            )],
        }
    }
    fn get(&self, view: &mut View, call: ToolCall, now: &Zoned) -> Vec<Effect> {
        let args: tools::List = match tool_args::parse(&call) {
            Ok(args) => args,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        let tz = now.time_zone();
        let bound = |value: Option<&String>| match value {
            None => Ok(None),
            Some(value) => instant(value, tz).map(Some).ok_or(()),
        };
        let (Ok(start), Ok(end)) = (bound(args.start.as_ref()), bound(args.end.as_ref())) else {
            return vec![Effect::Reply(
                call,
                Err(tool_args::error("Invalid time range")),
            )];
        };
        let events: Vec<_> = self
            .events
            .iter()
            .filter(|event| {
                start.is_none_or(|start| instant(&event.end, tz).is_none_or(|end| end > start))
                    && end.is_none_or(|end| instant(&event.start, tz).is_none_or(|s| s < end))
            })
            .cloned()
            .collect();
        let mut effects = match args.focus {
            Some(focus) => view.open(
                Screen::Calendar,
                Some(Focus::Calendar(focus)),
                now.timestamp().as_second(),
            ),
            None => Vec::new(),
        };
        effects.push(Effect::Reply(
            call,
            Ok(serde_json::json!({"events": events})),
        ));
        effects
    }
    fn create(&mut self, view: &mut View, call: ToolCall, now: &Zoned) -> Vec<Effect> {
        let args: tools::Create = match tool_args::parse(&call) {
            Ok(args) => args,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        if args.summary.trim().is_empty() || !valid_range(&args.start, &args.end) {
            let error = tool_args::error("Invalid event summary or time range");
            return vec![Effect::Reply(call, Err(error))];
        }
        let event = NewCalendarEvent {
            location: args.location,
            summary: args.summary,
            start: args.start,
            end: args.end,
        };
        let selected_day = if !event.start.contains('T') {
            event.start.parse::<Date>().map(|date| date.to_string())
        } else {
            event.start.parse::<Timestamp>().map(|timestamp| {
                timestamp
                    .to_zoned(now.time_zone().clone())
                    .date()
                    .to_string()
            })
        };
        let job = format!("{CREATE_JOB_PREFIX}{}", uuid::Uuid::new_v4());
        self.pending.push(CalendarPendingEvent {
            key: job.clone(),
            summary: event.summary.clone(),
            location: event.location.clone(),
            start: event.start.clone(),
            end: event.end.clone(),
            state: CalendarPendingState::Saving,
            saved_at: None,
        });
        let source = self.source.clone();
        let settings = self.settings.clone();
        let mut effects = selected_day
            .ok()
            .map(|day| {
                view.open(
                    Screen::Calendar,
                    Some(Focus::Calendar(day)),
                    now.timestamp().as_second(),
                )
            })
            .unwrap_or_default();
        effects.extend(self.snapshot());
        effects.push(Effect::Ui(Box::new(|ui| {
            ui.events.spawn = ui.events.spawn.saturating_add(1);
            true
        })));
        effects.push(Effect::Spawn {
            job,
            work: Box::pin(async move { Ok(Box::new(source.create(settings, event).await?) as _) }),
            reply: Some(call),
            timeout: Duration::from_secs(30),
        });
        effects
    }
    pub fn on_job(
        &mut self,
        id: &str,
        result: JobResult,
        reply: Option<ToolCall>,
        view: &mut View,
        now: &Zoned,
    ) -> Vec<Effect> {
        if id == SYNC_JOB {
            return self.synced(result, now);
        }
        let Some(call) = reply else {
            return vec![];
        };
        let timed_out = result
            .as_ref()
            .is_err_and(|error| error.to_string() == "Job timed out");
        let event = result.and_then(|value| {
            value
                .downcast::<CalendarEvent>()
                .map(|v| *v)
                .map_err(|_| anyhow::anyhow!("Invalid calendar result"))
        });
        let Ok(event) = event else {
            if let Some(pending) = self.pending.iter_mut().find(|pending| pending.key == id) {
                pending.state = CalendarPendingState::Failed;
                pending.saved_at = None;
            }
            let mut effects = self.snapshot();
            effects.push(Effect::ReplyError(
                call,
                if timed_out {
                    "Job timed out"
                } else {
                    "Calendar request failed"
                },
            ));
            return effects;
        };
        if let Some(pending) = self.pending.iter_mut().find(|pending| pending.key == id) {
            pending.key = event.id.clone();
            pending.summary = event.summary.clone();
            pending.location = event.location.clone();
            pending.start = event.start.clone();
            pending.end = event.end.clone();
            pending.state = CalendarPendingState::Saved;
            pending.saved_at = Some(now.timestamp().as_second());
        }
        self.created.push(event.id.clone());
        self.upsert(event.clone(), now);
        let mut effects = self.changed();
        effects.extend(view.open(
            Screen::Calendar,
            Some(Focus::Calendar(event.id.clone())),
            now.timestamp().as_second(),
        ));
        effects.push(Effect::Reply(call, Ok(serde_json::json!({"event": event}))));
        effects.extend(self.sync(false, now));
        effects
    }
    fn synced(&mut self, result: JobResult, now: &Zoned) -> Vec<Effect> {
        let delta = result.and_then(|value| {
            value
                .downcast::<(CalendarDelta, bool)>()
                .map_err(|_| anyhow::anyhow!("Invalid calendar result"))
        });
        let Ok(delta) = delta else {
            self.online = Some(false);
            return self.snapshot();
        };
        let (delta, full) = *delta;
        if full {
            self.last_full_day = Some(now.date());
            self.events.retain(|event| self.created.contains(&event.id));
        }
        self.events
            .retain(|event| !delta.removed.contains(&event.id));
        for event in delta.events {
            self.created.retain(|id| id != &event.id);
            self.upsert(event, now);
        }
        self.token = delta.sync_token;
        self.last_sync = Some(now.timestamp().as_second());
        self.online = Some(true);
        self.prune(now);
        self.changed()
    }
    /// Keeps events inside the cached window, and events created here.
    fn prune(&mut self, now: &Zoned) {
        let Some((from, to)) = window(now) else {
            return;
        };
        let tz = now.time_zone();
        self.events.retain(|event| {
            self.created.contains(&event.id)
                || (instant(&event.end, tz).is_some_and(|end| end >= from)
                    && instant(&event.start, tz).is_some_and(|start| start <= to))
        });
    }
    fn upsert(&mut self, event: CalendarEvent, now: &Zoned) {
        self.events.retain(|old| old.id != event.id);
        self.events.push(event);
        let tz = now.time_zone();
        self.events
            .sort_by_cached_key(|event| instant(&event.start, tz));
    }
    fn changed(&self) -> Vec<Effect> {
        let mut effects = self.snapshot();
        let events = self.events.clone();
        let token = self.token.clone();
        let last_sync = self.last_sync;
        effects.push(Effect::store(move |store| {
            store.calendar_save(&events, &token)?;
            store.calendar_save_sync(last_sync)
        }));
        effects
    }
}
