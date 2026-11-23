use crate::{ImageFetcher, runtime::effects::Effect};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Photo {
    pub id: String,
    pub caption: String,
    pub source: String,
    pub image_path: String,
    pub thumbnail_path: String,
    pub created: i64,
    pub pinned: bool,
}
pub(crate) struct Feature {
    source: Arc<dyn ImageFetcher>,
    photos: Vec<Photo>,
}
impl Feature {
    pub fn load(source: Arc<dyn ImageFetcher>, store: &crate::Store) -> anyhow::Result<Self> {
        Ok(Self {
            source,
            photos: store.photos_load()?,
        })
    }
    pub fn snapshot(&self) -> Effect {
        let photos = self.photos.clone();
        Effect::Ui(Box::new(move |ui| {
            if ui.photos == photos {
                false
            } else {
                ui.photos = photos;
                true
            }
        }))
    }
    pub fn on_tool(
        &self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: i64,
        store: &crate::Store,
    ) -> Vec<Effect> {
        if call.name == "hub_image_list" || call.name == "hub_image_control" {
            return self.cached_tool(view, call, now, store);
        }
        let args: super::tools::Show = match serde_json::from_value(call.arguments.clone()) {
            Ok(args) => args,
            Err(_) => return vec![Effect::ReplyError(call, "Invalid image arguments")],
        };
        if args.images.is_empty() || args.images.len() > 6 {
            return vec![Effect::ReplyError(call, "Expected one to six images")];
        }
        let source = self.source.clone();
        let requests: Vec<_> = args
            .images
            .into_iter()
            .map(|input| {
                let source = url::Url::parse(&input.url)
                    .ok()
                    .and_then(|url| url.host_str().map(str::to_owned))
                    .unwrap_or_default();
                Request {
                    url: input.url,
                    fetch: crate::PhotoFetch {
                        key: uuid::Uuid::new_v4().to_string(),
                        caption: input.caption,
                        source,
                        created: now,
                        state: crate::PhotoFetchState::Loading,
                    },
                }
            })
            .collect();
        let pending: Vec<_> = requests
            .iter()
            .map(|request| request.fetch.clone())
            .collect();
        let mut effects = vec![Effect::Ui(Box::new(move |ui| {
            ui.photo_fetches
                .retain(|photo| photo.state != crate::PhotoFetchState::Loading);
            ui.photo_fetches.extend(pending);
            true
        }))];
        effects.push(Effect::Spawn {
            job: "photos".into(),
            reply: Some(call),
            timeout: Duration::from_secs(120),
            work: Box::pin(async move {
                let mut results = Vec::new();
                for request in requests {
                    results.push((request.fetch, source.fetch(request.url).await));
                }
                Ok(Box::new(results) as Box<dyn std::any::Any + Send>)
            }),
        });
        effects
    }
    fn cached_tool(
        &self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: i64,
        store: &crate::Store,
    ) -> Vec<Effect> {
        let mut effects = Vec::new();
        let photos = match store.photos_load() {
            Ok(photos) => photos,
            Err(_) => {
                effects.push(Effect::ReplyError(call, "Images unavailable"));
                return effects;
            }
        };
        if call.name == "hub_image_list" {
            if serde_json::from_value::<super::tools::List>(call.arguments.clone()).is_err() {
                effects.push(Effect::ReplyError(call, "Invalid image arguments"));
                return effects;
            }
            effects.push(Effect::Reply(
                call,
                Ok(serde_json::json!({"images": photos})),
            ));
            return effects;
        }
        let args: super::tools::Control = match serde_json::from_value(call.arguments.clone()) {
            Ok(args) => args,
            Err(_) => {
                effects.push(Effect::ReplyError(call, "Invalid image arguments"));
                return effects;
            }
        };
        let selected = args.id.as_deref().or_else(|| match view.focus() {
            Some(crate::Focus::Photo(id)) => Some(id.as_str()),
            _ => None,
        });
        let index = match selected {
            Some(id) => photos.iter().position(|p| p.id == id),
            None if !photos.is_empty() => Some(0),
            _ => None,
        };
        let Some(index) = index else {
            effects.push(Effect::ReplyError(call, "Image not found"));
            return effects;
        };
        let index = match args.action {
            super::tools::Action::Next => (index + 1) % photos.len(),
            super::tools::Action::Previous => (index + photos.len() - 1) % photos.len(),
            _ => index,
        };
        let id = photos[index].id.clone();
        match args.action {
            super::tools::Action::Pin => effects.push(Effect::PinPhoto(id.clone(), true)),
            super::tools::Action::Unpin => effects.push(Effect::PinPhoto(id.clone(), false)),
            _ => {}
        }
        effects.extend(view.open(
            crate::Screen::Photos,
            Some(crate::Focus::Photo(id.clone())),
            now,
        ));
        effects.push(Effect::Reply(
            call,
            Ok(serde_json::json!({"ok": true, "id": id})),
        ));
        effects
    }
    pub fn on_job(
        &mut self,
        result: crate::runtime::jobs::JobResult,
        reply: Option<ToolCall>,
        now: i64,
    ) -> Vec<Effect> {
        let result = result.and_then(|r| {
            r.downcast::<Vec<(crate::PhotoFetch, anyhow::Result<crate::FetchedImage>)>>()
                .map_err(|_| anyhow::anyhow!("invalid photo result"))
        });
        match result {
            Ok(results) => vec![Effect::PersistPhotos(*results, now, reply)],
            Err(_) => {
                let mut effects = vec![Effect::Ui(Box::new(|ui| {
                    for photo in &mut ui.photo_fetches {
                        if photo.state == crate::PhotoFetchState::Loading {
                            photo.state = crate::PhotoFetchState::Failed;
                        }
                    }
                    true
                }))];
                effects.extend(
                    reply
                        .map(|call| Effect::ReplyError(call, "Images unavailable"))
                        .into_iter(),
                );
                effects
            }
        }
    }
}

struct Request {
    fetch: crate::PhotoFetch,
    url: String,
}
