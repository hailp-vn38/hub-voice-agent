mod store;
pub(crate) mod tools;
use crate::{ImageFetcher, runtime::effects::Effect};
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Photo {
    pub id: String,
    pub caption: String,
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
    pub fn on_tool(&self, view: &mut crate::view::View, call: ToolCall, now: i64) -> Vec<Effect> {
        let args: tools::Show = match serde_json::from_value(call.arguments.clone()) {
            Ok(args) => args,
            Err(_) => return vec![Effect::ReplyError(call, "Invalid image arguments")],
        };
        if args.images.is_empty() || args.images.len() > 6 {
            return vec![Effect::ReplyError(call, "Expected one to six images")];
        }
        let source = self.source.clone();
        let mut effects = view.open(crate::Screen::Photos, None, now);
        effects.push(Effect::Spawn {
            job: "photos".into(),
            reply: Some(call),
            timeout: Duration::from_secs(120),
            work: Box::pin(async move {
                let mut results = Vec::new();
                for input in args.images {
                    results.push((input.caption, source.fetch(input.url).await));
                }
                Ok(Box::new(results) as Box<dyn std::any::Any + Send>)
            }),
        });
        effects
    }
    pub fn on_job(
        &mut self,
        result: crate::runtime::jobs::JobResult,
        reply: Option<ToolCall>,
        now: i64,
    ) -> Vec<Effect> {
        let result = result.and_then(|r| {
            r.downcast::<Vec<(String, anyhow::Result<crate::FetchedImage>)>>()
                .map_err(|_| anyhow::anyhow!("invalid photo result"))
        });
        match result {
            Ok(results) => vec![Effect::PersistPhotos(*results, now, reply)],
            Err(_) => reply
                .map(|call| Effect::ReplyError(call, "Images unavailable"))
                .into_iter()
                .collect(),
        }
    }
}
