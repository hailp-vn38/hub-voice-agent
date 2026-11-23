use super::{
    audio, effects::Effect, jobs::Internal, tick, tool_trail, tool_trail::ToolTrail, voice,
};
use crate::{
    Alert, AudioEvent, Clock, Config, HomeEvent, Ports, Screen, Store, System, ToolTrailState,
    UserInput,
    alerts::Alerts,
    conversation::{Conversation, Input},
    features::{calendar, countdowns, home, photos, reminders, timers, weather},
    view::{self, View},
};
use jiff::Zoned;
use std::sync::Arc;
use voice_client::{ToolState, VoiceEvent};

/// Everything the actor owns between events. Each handler returns the effects
/// to execute.
pub(super) struct Hub {
    clock: Arc<dyn Clock>,
    system: Arc<dyn System>,
    pub(super) timers: timers::Feature,
    pub(super) countdowns: countdowns::Feature,
    pub(super) reminders: reminders::Feature,
    speech: reminders::speech::Speech,
    pub(super) calendar: calendar::Feature,
    pub(super) weather: weather::Feature,
    pub(super) home: home::Feature,
    pub(super) photos: photos::Feature,
    conversation: Conversation,
    pub(super) alerts: Alerts,
    pub(super) view: View,
    tool_trail: ToolTrail,
}

impl Hub {
    pub fn load(config: &Config, store: &mut Store, ports: &Ports) -> anyhow::Result<Self> {
        let now = ports.clock.now().timestamp().as_second();
        Ok(Self {
            clock: ports.clock.clone(),
            system: ports.system.clone(),
            timers: timers::Feature::load(store, now)?,
            countdowns: countdowns::Feature::load(store)?,
            reminders: reminders::Feature::load(store)?,
            speech: Default::default(),
            calendar: calendar::Feature::load(
                ports.calendar.clone(),
                config.calendar.clone(),
                store,
            )?,
            weather: weather::Feature::load(ports.weather.clone(), &config.weather, store, now)?,
            home: home::Feature::new(ports.home.clone()),
            photos: photos::Feature::load(ports.images.clone(), store)?,
            conversation: Conversation::default(),
            alerts: Alerts::default(),
            view: View::new(now),
            tool_trail: ToolTrail::default(),
        })
    }

    pub fn now(&self) -> Zoned {
        self.clock.now()
    }

    /// The first UI snapshot from the loaded state.
    pub fn snapshot(&self) -> Vec<Effect> {
        let mut effects = vec![self.weather.snapshot(), self.photos.snapshot()];
        effects.extend(self.timers.snapshot());
        effects.extend(self.countdowns.snapshot(self.now().date()));
        effects.extend(self.reminders.snapshot());
        effects.extend(self.calendar.snapshot());
        effects
    }

    /// Runs `handle`, then drops a weather preview if it moved the view off Weather.
    pub fn track_view(
        &mut self,
        store: &Store,
        handle: impl FnOnce(&mut Self) -> Vec<Effect>,
    ) -> Vec<Effect> {
        let old = self.view.screen();
        let mut effects = handle(self);
        if old == Screen::Weather && self.view.screen() != Screen::Weather {
            let now = self.now().timestamp().as_second();
            effects.extend(self.weather.leave(store, now));
        }
        effects
    }

    pub fn tick(&mut self, store: &Store, config: &Config) -> Vec<Effect> {
        let now = self.now();
        let seconds = now.timestamp().as_second();
        let synced = self.system.time_synced();
        let mut effects = tick::tick(&now);
        effects.extend(self.conversation.on(Input::Tick, now.clone()));
        effects.extend(self.countdowns.snapshot(now.date()));
        effects.extend(self.track_view(store, |hub| hub.view.tick(seconds)));
        effects.extend(self.weather.tick(seconds));
        let due_timers = self.timers.due(seconds);
        let due_reminders = self.reminders.due(&now);
        self.alerts.retain_due(&due_timers, &due_reminders);
        for timer in due_timers {
            self.alerts.push_timer(timer);
        }
        if synced {
            effects.extend(self.reminders.mark_missed(&now));
        }
        for reminder in due_reminders {
            self.alerts.push_reminder(reminder);
        }
        let blocked = self.conversation.alert_blocked();
        let (dismissed, alert_effects) = self.alerts.tick(seconds, blocked, synced);
        effects.extend(alert_effects);
        let speak = config.voice.reminder_speech;
        effects.extend(self.speech.tick(self.alerts.active(), seconds, speak));
        effects.extend(self.finish_alert(dismissed, &now));
        effects.extend(self.calendar.tick(&now));
        effects
    }

    pub fn dismiss_alert(&mut self) -> Vec<Effect> {
        let (dismissed, mut effects) = self.alerts.dismiss();
        effects.extend(self.finish_alert(dismissed, &self.now()));
        effects
    }

    fn finish_alert(&mut self, alert: Option<Alert>, now: &Zoned) -> Vec<Effect> {
        match alert {
            Some(Alert::Timer(timer)) => self.timers.dismiss(timer.id),
            Some(Alert::Reminder(reminder)) => self.reminders.finish(reminder.id, now, false),
            None => Vec::new(),
        }
    }

    pub fn on_job(&mut self, event: Internal, store: &Store) -> Vec<Effect> {
        let now = self.now();
        let seconds = now.timestamp().as_second();
        let (id, result, reply) = match event {
            Internal::JobDone(id, result, reply) => (id, result, reply),
            Internal::Timeout(id, reply) if id == "weather" => {
                (id, Err(anyhow::anyhow!("Job timed out")), reply)
            }
            Internal::Timeout(_, reply) => {
                let tool_name = reply.as_ref().map(|call| call.name.clone());
                let mut effects = reply
                    .map(|call| Effect::ReplyError(call, "Job timed out"))
                    .into_iter()
                    .collect::<Vec<_>>();
                if let Some(name) = tool_name
                    && let Some(effect) =
                        self.tool_trail.finish_latest(&name, ToolTrailState::Failed)
                {
                    effects.insert(0, effect);
                }
                return effects;
            }
        };
        let tool_name = reply.as_ref().map(|call| call.name.clone());
        let mut effects = self.track_view(store, |hub| {
            if id == "weather" {
                hub.weather.on_job(result, reply, seconds)
            } else if calendar::Feature::owns_job(&id) {
                hub.calendar.on_job(&id, result, reply, &mut hub.view, &now)
            } else if id == "photos" {
                hub.photos.on_job(result, reply, seconds)
            } else if id.starts_with("home:") {
                hub.home.on_job(result, reply)
            } else {
                Vec::new()
            }
        });
        if let Some(name) = tool_name
            && let Some(state) = tool_trail::outcome(&effects)
            && let Some(effect) = self.tool_trail.finish_latest(&name, state)
        {
            effects.insert(0, effect);
        }
        effects
    }

    pub fn on_voice(&mut self, event: VoiceEvent, store: &Store) -> Vec<Effect> {
        let now = self.now();
        self.speech.event(&event, now.timestamp().as_second());
        self.track_view(store, |hub| match event {
            VoiceEvent::ToolCall(call) => {
                let name = call.name.clone();
                let (key, started) = hub.tool_trail.start(name);
                let tool_effects = hub.on_tool(call, store);
                let mut effects = vec![started];
                if let Some(state) = tool_trail::outcome(&tool_effects)
                    && let Some(effect) = hub.tool_trail.finish(&key, state)
                {
                    effects.push(effect);
                }
                effects.extend(tool_effects);
                effects
            }
            VoiceEvent::ServerTool { name, state } => {
                let seconds = now.timestamp().as_second();
                match state {
                    ToolState::Started => {
                        let (_, started) = hub.tool_trail.start(name.clone());
                        let mut effects = view::screen_for_server_tool(&name)
                            .map(|screen| hub.view.show(screen, seconds))
                            .unwrap_or_default();
                        effects.push(started);
                        effects
                    }
                    ToolState::Completed => hub
                        .tool_trail
                        .finish_latest(&name, ToolTrailState::Done)
                        .into_iter()
                        .collect(),
                    ToolState::Failed => hub
                        .tool_trail
                        .finish_latest(&name, ToolTrailState::Failed)
                        .into_iter()
                        .collect(),
                }
            }
            event => voice::on(event, &mut hub.conversation, now),
        })
    }

    pub fn on_audio(&mut self, event: AudioEvent) -> Vec<Effect> {
        let now = self.now();
        let mut effects = Vec::new();
        let starts_turn = matches!(&event, AudioEvent::WakeWord) && self.conversation.is_idle();
        if matches!(&event, AudioEvent::WakeWord) {
            effects.extend(
                self.view
                    .on_user(UserInput::Activity, now.timestamp().as_second()),
            );
        }
        if starts_turn && let Some(effect) = self.tool_trail.clear() {
            effects.push(effect);
        }
        effects.extend(audio::on(event, &mut self.conversation, now));
        effects
    }

    pub fn on_home(&mut self, event: HomeEvent) -> Vec<Effect> {
        self.home.event(event)
    }

    pub fn on_input(&mut self, input: UserInput, store: &Store) -> Vec<Effect> {
        if matches!(&input, UserInput::OrbTapped) {
            return self.on_orb_tapped(store);
        }
        let now = self.now().timestamp().as_second();
        self.track_view(store, |hub| match input {
            UserInput::Timer(input) => hub.timers.on_input(&mut hub.view, input, now),
            input => hub.view.on_user(input, now),
        })
    }

    fn on_orb_tapped(&mut self, store: &Store) -> Vec<Effect> {
        let now = self.now();
        let starts_turn = self.conversation.is_idle();
        self.track_view(store, |hub| {
            let mut effects = hub
                .view
                .on_user(UserInput::Activity, now.timestamp().as_second());
            if starts_turn && let Some(effect) = hub.tool_trail.clear() {
                effects.push(effect);
            }
            effects.extend(hub.conversation.on(Input::OrbTapped, now.clone()));
            effects
        })
    }
}
