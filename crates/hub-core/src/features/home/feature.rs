use crate::{HomeEntity, HomeEvent, HomeSource, runtime::effects::Effect};
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
    pub fn on_tool(&self, view: &mut crate::view::View, call: ToolCall, now: i64) -> Vec<Effect> {
        let mut effects = view.open(crate::Screen::Home, None, now);
        let args: super::tools::Args = match serde_json::from_value(call.arguments.clone()) {
            Ok(args) => args,
            Err(e) => {
                effects.push(Effect::Reply(
                    call,
                    Err(crate::features::countdowns::tools::error(&format!(
                        "Invalid arguments: {e}"
                    ))),
                ));
                return effects;
            }
        };
        let names: Vec<_> = self
            .entities
            .iter()
            .map(|entity| entity.name.as_str())
            .collect();
        let name = match crate::selector::select(&names, &args.entity) {
            crate::selector::Selection::Found(name) => name,
            crate::selector::Selection::Ambiguous(names) => {
                effects.push(Effect::Reply(
                    call,
                    Err(crate::features::countdowns::tools::error(&format!(
                        "Ambiguous entity: {}",
                        names.join(", ")
                    ))),
                ));
                return effects;
            }
            crate::selector::Selection::NotFound => {
                effects.push(Effect::Reply(
                    call,
                    Err(crate::features::countdowns::tools::error(
                        "Entity not found",
                    )),
                ));
                return effects;
            }
        };
        let entity = self
            .entities
            .iter()
            .find(|entity| entity.name == name)
            .unwrap();
        if !self.connected {
            effects.push(Effect::ReplyError(call, "Home Assistant disconnected"));
            return effects;
        }
        if call.name == "hub_home_get" {
            effects.push(Effect::Reply(
                call,
                Ok(serde_json::to_value(entity).unwrap()),
            ));
            return effects;
        }
        let service = match call.name.as_str() {
            "hub_home_turn_on" => "turn_on",
            "hub_home_turn_off" => "turn_off",
            _ => {
                effects.push(Effect::ReplyError(call, "Unknown Home Assistant tool"));
                return effects;
            }
        };
        let source = self.source.clone();
        let entity_id = entity.id.clone();
        let job = format!("home:{entity_id}");
        effects.push(Effect::Spawn {
            job,
            work: Box::pin(async move {
                source.call_service(entity_id, service.into()).await?;
                Ok(Box::new(()) as Box<dyn std::any::Any + Send>)
            }),
            reply: Some(call),
            timeout: Duration::from_secs(10),
        });
        effects
    }
}
