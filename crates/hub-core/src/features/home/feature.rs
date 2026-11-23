use super::tools::Args;
use crate::{
    HomeEntity, HomeEvent, HomeSource,
    features::tool_args,
    runtime::{effects::Effect, jobs::JobResult},
    selector::{self, Selection},
};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

pub(crate) struct Feature {
    source: Arc<dyn HomeSource>,
    connected: bool,
    entities: Vec<HomeEntity>,
}
impl Feature {
    pub fn new(source: Arc<dyn HomeSource>) -> Self {
        Self {
            source,
            connected: false,
            entities: Vec::new(),
        }
    }
    fn snapshot(&self) -> Effect {
        let entities = self.entities.clone();
        Effect::Ui(Box::new(move |ui| {
            if ui.home == entities {
                false
            } else {
                ui.home = entities;
                true
            }
        }))
    }
    pub fn event(&mut self, event: HomeEvent) -> Vec<Effect> {
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
    pub fn on_tool(&self, call: ToolCall) -> Vec<Effect> {
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
        let source = self.source.clone();
        let entity_id = entity.id.clone();
        vec![Effect::Spawn {
            job: format!("home:{entity_id}"),
            work: Box::pin(async move {
                source.call_service(entity_id, service.into()).await?;
                Ok(Box::new(()) as Box<dyn std::any::Any + Send>)
            }),
            reply: Some(call),
            timeout: Duration::from_secs(10),
        }]
    }
    pub fn on_job(&self, result: JobResult, reply: Option<ToolCall>) -> Vec<Effect> {
        let Some(call) = reply else {
            return Vec::new();
        };
        vec![match result {
            Ok(_) => Effect::Reply(call, Ok(serde_json::json!({"ok": true}))),
            Err(e) => Effect::Reply(call, Err(tool_args::error(&e.to_string()))),
        }]
    }
}
