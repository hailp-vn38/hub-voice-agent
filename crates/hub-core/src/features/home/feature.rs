use super::tools::Args;
use crate::{
    HomeDomain, HomeEntity, HomeEvent, HomeSource,
    features::tool_args,
    runtime::{effects::Effect, jobs::JobResult},
    selector::{self, Selection},
};
use std::collections::HashSet;
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

pub(crate) struct Feature {
    source: Arc<dyn HomeSource>,
    connected: bool,
    configured: bool,
    entities: Vec<HomeEntity>,
    pending: HashSet<String>,
    last_updated: Option<i64>,
}
impl Feature {
    pub fn new(source: Arc<dyn HomeSource>, settings: &crate::HomeSettings) -> Self {
        Self {
            source,
            connected: false,
            configured: configured(settings),
            entities: Vec::new(),
            pending: HashSet::new(),
            last_updated: None,
        }
    }
    pub fn snapshot(&self) -> Effect {
        let entities = self.entities.clone();
        let connected = self.connected;
        let configured = self.configured;
        let pending = self.pending.iter().cloned().collect::<Vec<_>>();
        let last_updated = self.last_updated;
        Effect::Ui(Box::new(move |ui| {
            let changed = ui.home != entities
                || ui.home_connected != connected
                || ui.home_configured != configured
                || ui.home_pending != pending
                || ui.home_last_updated != last_updated;
            if !changed {
                false
            } else {
                ui.home = entities;
                ui.home_connected = connected;
                ui.home_configured = configured;
                ui.home_pending = pending;
                ui.home_last_updated = last_updated;
                true
            }
        }))
    }
    pub fn settings_changed(&mut self, settings: &crate::HomeSettings) -> Vec<Effect> {
        self.configured = configured(settings);
        vec![self.snapshot()]
    }
    pub fn event(&mut self, event: HomeEvent, now: i64) -> Vec<Effect> {
        match event {
            HomeEvent::Connected => self.connected = true,
            HomeEvent::Disconnected => {
                self.connected = false;
                for entity in &mut self.entities {
                    entity.stale = true;
                }
            }
            HomeEvent::State(mut entity) => {
                entity.stale = false;
                self.pending.remove(&entity.id);
                self.last_updated = Some(now);
                if let Some(old) = self.entities.iter_mut().find(|e| e.id == entity.id) {
                    *old = entity;
                } else {
                    self.entities.push(entity);
                }
            }
        }
        vec![self.snapshot()]
    }
    /// `hub_home_get` answers from the last known state, marked stale while
    /// disconnected; turning a device on or off needs a live connection.
    pub fn on_tool(&mut self, call: ToolCall) -> Vec<Effect> {
        let args: Args = match tool_args::parse(&call) {
            Ok(args) => args,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        let service = match call.name.as_str() {
            "hub_home_get" => None,
            "hub_home_turn_on" => Some("turn_on"),
            "hub_home_turn_off" => Some("turn_off"),
            _ => return vec![Effect::ReplyError(call, "Unknown Home Assistant tool")],
        };
        if service.is_some() && !self.connected {
            return vec![Effect::ReplyError(call, "Home Assistant disconnected")];
        }
        let names: Vec<_> = self.entities.iter().map(|e| e.name.as_str()).collect();
        let entity = match selector::select(&names, &args.entity) {
            Selection::Found(index) => &self.entities[index],
            Selection::NotFound if !self.connected => {
                return vec![Effect::ReplyError(call, "Home Assistant disconnected")];
            }
            Selection::NotFound => {
                return vec![Effect::Reply(
                    call,
                    Err(tool_args::error("Entity not found")),
                )];
            }
            Selection::Ambiguous(names) => {
                let message = format!("Ambiguous entity: {}", names.join(", "));
                return vec![Effect::Reply(call, Err(tool_args::error(&message)))];
            }
        };
        let Some(service) = service else {
            let state = serde_json::to_value(entity).expect("entity serializes");
            return vec![Effect::Reply(call, Ok(state))];
        };
        let entity_id = entity.id.clone();
        self.call_service(entity_id, service.into(), Some(call))
    }
    pub fn toggle(&mut self, entity_id: &str) -> Vec<Effect> {
        if !self.connected {
            return Vec::new();
        }
        let Some(entity) = self.entities.iter().find(|entity| entity.id == entity_id) else {
            return Vec::new();
        };
        if !matches!(entity.domain(), HomeDomain::Light | HomeDomain::Switch) {
            return Vec::new();
        }
        let service = if entity.state == "on" {
            "turn_off"
        } else {
            "turn_on"
        };
        self.call_service(entity_id.to_owned(), service.into(), None)
    }
    fn call_service(
        &mut self,
        entity_id: String,
        service: String,
        reply: Option<ToolCall>,
    ) -> Vec<Effect> {
        if !self.connected {
            return reply
                .map(|call| Effect::ReplyError(call, "Home Assistant disconnected"))
                .into_iter()
                .collect();
        }
        self.pending.insert(entity_id.clone());
        let source = self.source.clone();
        let mut effects = vec![self.snapshot()];
        effects.push(Effect::Spawn {
            job: format!("{}{entity_id}", super::JOB_PREFIX),
            work: Box::pin(async move {
                source.call_service(entity_id, service).await?;
                Ok(Box::new(()) as Box<dyn std::any::Any + Send>)
            }),
            reply,
            timeout: Duration::from_secs(10),
        });
        effects
    }
    pub fn on_job(&mut self, id: &str, result: JobResult, reply: Option<ToolCall>) -> Vec<Effect> {
        let mut effects = Vec::new();
        if result.is_err()
            && let Some(entity_id) = id.strip_prefix(super::JOB_PREFIX)
            && self.pending.remove(entity_id)
        {
            effects.push(self.snapshot());
        }
        let Some(call) = reply else {
            return effects;
        };
        effects.push(match result {
            Ok(_) => Effect::Reply(call, Ok(serde_json::json!({"ok": true}))),
            Err(e) => Effect::Reply(call, Err(tool_args::error(&e.to_string()))),
        });
        effects
    }
}

fn configured(settings: &crate::HomeSettings) -> bool {
    !settings.url.trim().is_empty() && !settings.token.trim().is_empty()
}
