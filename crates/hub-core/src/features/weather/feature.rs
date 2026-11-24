use super::model::WeatherForecast;
use crate::{WeatherLocation, WeatherSource, runtime::effects::Effect};
use serde_json::Value;
use std::{sync::Arc, time::Duration};
use voice_client::ToolCall;

/// How long a fetched forecast stays fresh.
const REFRESH_SECONDS: i64 = 1800;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherView {
    pub location: Option<String>,
    pub preview: bool,
    pub updated: Option<i64>,
    pub forecast: Option<WeatherForecast>,
    pub stale: bool,
}

pub(crate) struct Feature {
    source: Arc<dyn WeatherSource>,
    home: Option<(f64, f64)>,
    home_name: Option<String>,
    active: String,
    next_refresh: i64,
    pending: bool,
    view: WeatherView,
}
impl Feature {
    pub fn load(
        source: Arc<dyn WeatherSource>,
        settings: &crate::WeatherSettings,
        store: &crate::Store,
        now: i64,
    ) -> anyhow::Result<Self> {
        let home = settings.latitude.zip(settings.longitude);
        let active = Self::home_key(home);
        let mut feature = Self {
            source,
            home,
            home_name: settings.name.clone(),
            active,
            next_refresh: 0,
            pending: false,
            view: WeatherView::default(),
        };
        feature.restore(store, now)?;
        Ok(feature)
    }
    fn home_key(home: Option<(f64, f64)>) -> String {
        home.map(|(a, b)| format!("home:{a},{b}"))
            .unwrap_or_default()
    }
    /// Shows the cached forecast for the active location and schedules its
    /// refresh 30 minutes after it was fetched (now when nothing is cached).
    fn restore(&mut self, store: &crate::Store, now: i64) -> anyhow::Result<()> {
        self.view = WeatherView::default();
        self.next_refresh = now;
        if let Some((updated, data)) = store.weather_get(&self.active)? {
            self.apply(
                data,
                now.saturating_sub(updated) >= REFRESH_SECONDS,
                updated,
            );
            self.next_refresh = updated.saturating_add(REFRESH_SECONDS);
        }
        Ok(())
    }
    fn apply(&mut self, data: Value, stale: bool, updated: i64) {
        self.view = WeatherView {
            location: data["location"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| {
                    (!self.active.starts_with("city:"))
                        .then(|| self.home_name.clone())
                        .flatten()
                }),
            preview: self.active.starts_with("city:"),
            updated: Some(updated),
            forecast: Some(WeatherForecast::from_open_meteo(&data)),
            stale,
        };
    }
    /// Adds the location name to a reply so the model knows which place it is.
    fn named(&self, mut data: Value) -> Value {
        if data.get("location").is_none()
            && let (Some(map), Some(name)) = (data.as_object_mut(), self.view.location.clone())
        {
            map.insert("location".into(), Value::String(name));
        }
        data
    }
    pub fn snapshot(&self) -> Effect {
        let weather = self.view.clone();
        Effect::Ui(Box::new(move |ui| {
            if ui.weather == weather {
                false
            } else {
                ui.weather = weather;
                true
            }
        }))
    }
    fn spawn(&self, location: WeatherLocation, reply: Option<ToolCall>) -> Effect {
        let source = self.source.clone();
        let key = self.active.clone();
        Effect::Spawn {
            job: "weather".into(),
            work: Box::pin(async move {
                Ok(Box::new((key, source.forecast(location).await))
                    as Box<dyn std::any::Any + Send>)
            }),
            reply,
            timeout: Duration::from_secs(20),
        }
    }
    pub fn tick(&mut self, now: i64) -> Vec<Effect> {
        if self.pending || now < self.next_refresh {
            return vec![];
        }
        self.next_refresh = now.saturating_add(REFRESH_SECONDS);
        let location = if self.active.starts_with("city:") {
            Some(WeatherLocation::City(self.active[5..].to_owned()))
        } else {
            self.home
                .map(|(latitude, longitude)| WeatherLocation::Coordinates {
                    latitude,
                    longitude,
                })
        };
        if let Some(l) = location {
            self.pending = true;
            vec![self.spawn(l, None)]
        } else {
            vec![]
        }
    }
    pub fn on_tool(
        &mut self,
        view: &mut crate::view::View,
        call: ToolCall,
        now: i64,
        store: &crate::Store,
    ) -> Vec<Effect> {
        let args: super::tools::Get = match crate::features::tool_args::parse(&call) {
            Ok(args) => args,
            Err(e) => return vec![Effect::Reply(call, Err(e))],
        };
        let city = args.location.filter(|s| !s.trim().is_empty());
        self.active = city
            .as_ref()
            .map(|s| format!("city:{}", s.trim()))
            .unwrap_or_else(|| Self::home_key(self.home));
        let _ = self.restore(store, now);
        let mut effects = match args.focus {
            Some(focus) => view.open(
                crate::Screen::Weather,
                Some(crate::Focus::Weather(focus)),
                now,
            ),
            None => Vec::new(),
        };
        effects.push(self.snapshot());
        if let Some((_, data)) = store
            .weather_get(&self.active)
            .ok()
            .flatten()
            .filter(|(updated, _)| now.saturating_sub(*updated) < REFRESH_SECONDS)
        {
            self.pending = false;
            effects.push(Effect::CancelJob("weather".into()));
            effects.push(Effect::Reply(call, Ok(self.named(data))));
            return effects;
        }
        let location = city.map(WeatherLocation::City).or_else(|| {
            self.home
                .map(|(latitude, longitude)| WeatherLocation::Coordinates {
                    latitude,
                    longitude,
                })
        });
        match location {
            Some(location) => {
                self.pending = true;
                effects.push(self.spawn(location, Some(call)))
            }
            None => effects.push(Effect::ReplyError(call, "Home location not configured")),
        }
        effects
    }
    pub fn on_job(
        &mut self,
        result: crate::runtime::jobs::JobResult,
        reply: Option<ToolCall>,
        now: i64,
    ) -> Vec<Effect> {
        self.pending = false;
        let result = result.and_then(|value| {
            value
                .downcast::<(String, anyhow::Result<Value>)>()
                .map_err(|_| anyhow::anyhow!("invalid weather result"))
        });
        match result {
            Ok(result) => {
                let (key, data) = *result;
                match data {
                    Ok(data) => {
                        let saved = (key.clone(), data.clone());
                        let mut effects = vec![Effect::store(move |store| {
                            store.weather_save(&saved.0, now, &saved.1)
                        })];
                        if key == self.active {
                            self.apply(data.clone(), false, now);
                            effects.push(self.snapshot());
                        }
                        if let Some(call) = reply {
                            effects.push(Effect::Reply(call, Ok(self.named(data))));
                        }
                        effects
                    }
                    Err(_) => self.failed(reply, key == self.active),
                }
            }
            Err(_) => self.failed(reply, true),
        }
    }
    fn failed(&mut self, reply: Option<ToolCall>, active: bool) -> Vec<Effect> {
        let mut effects = Vec::new();
        if active {
            self.view.stale = true;
            effects.push(self.snapshot());
        }
        if let Some(call) = reply {
            effects.push(Effect::ReplyError(call, "Weather unavailable"));
        }
        effects
    }

    pub fn leave(&mut self, store: &crate::Store, now: i64) -> Vec<Effect> {
        if !self.active.starts_with("city:") {
            return vec![];
        }
        self.active = Self::home_key(self.home);
        let _ = self.restore(store, now);
        vec![self.snapshot()]
    }
    pub fn update_home(
        &mut self,
        settings: &crate::WeatherSettings,
        store: &crate::Store,
        now: i64,
    ) -> Vec<Effect> {
        self.home = settings.latitude.zip(settings.longitude);
        self.home_name = settings.name.clone();
        self.active = Self::home_key(self.home);
        let _ = self.restore(store, now);
        vec![self.snapshot()]
    }
}
