use crate::CalendarSettings;
use jiff::Zoned;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

/// Source of the current time. Time always enters hub-core through this port.
pub trait Clock: Send + Sync {
    fn now(&self) -> Zoned;
}

/// Wall clock in the system time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Zoned {
        Zoned::now()
    }
}

/// A weather request resolves a named city or the configured coordinates.
#[derive(Clone, Debug)]
pub enum WeatherLocation {
    City(String),
    Coordinates { latitude: f64, longitude: f64 },
}

pub trait WeatherSource: Send + Sync {
    fn forecast(
        &self,
        location: WeatherLocation,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send + '_>,
    >;
}

/// Reports whether wall time is reliable.
pub trait System: Send + Sync {
    fn time_synced(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct HomeEntity {
    pub id: String,
    pub name: String,
    pub state: String,
    pub unit: Option<String>,
    pub stale: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeDomain {
    Light,
    Switch,
    Sensor,
}

impl HomeEntity {
    pub(crate) fn domain(&self) -> HomeDomain {
        match self.id.split_once('.').map(|(domain, _)| domain) {
            Some("light") => HomeDomain::Light,
            Some("switch") => HomeDomain::Switch,
            _ => HomeDomain::Sensor,
        }
    }
}

#[derive(Clone, Debug)]
pub enum HomeEvent {
    Connected,
    State(HomeEntity),
    Disconnected,
}

pub trait HomeSource: Send + Sync {
    fn subscribe(
        &self,
        settings: tokio::sync::watch::Receiver<crate::HomeSettings>,
    ) -> tokio::sync::mpsc::Receiver<HomeEvent>;
    fn call_service(
        &self,
        entity_id: String,
        service: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + Send + '_>>;
}

/// Decoded, size-limited PNG data for display and its preview.
#[derive(Clone, Debug)]
pub struct FetchedImage {
    pub image: Vec<u8>,
    pub thumbnail: Vec<u8>,
}

pub trait ImageFetcher: Send + Sync {
    fn fetch(
        &self,
        url: String,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = anyhow::Result<FetchedImage>> + Send + '_>,
    >;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    #[serde(default)]
    pub location: Option<String>,
    pub id: String,
    pub summary: String,
    pub start: String,
    pub end: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewCalendarEvent {
    pub location: Option<String>,
    pub summary: String,
    pub start: String,
    pub end: String,
}
#[derive(Default, Debug)]
pub struct CalendarDelta {
    pub events: Vec<CalendarEvent>,
    pub removed: Vec<String>,
    pub sync_token: Option<String>,
    pub expired: bool,
}
pub type CalendarFuture<T> = Pin<Box<dyn Future<Output = anyhow::Result<T>> + Send>>;
/// What a calendar listing asks for: a full window or the changes since a sync token.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CalendarQuery {
    Full {
        time_min: jiff::Timestamp,
        time_max: jiff::Timestamp,
    },
    Since(String),
}
pub trait CalendarSource: Send + Sync {
    fn list(
        &self,
        settings: CalendarSettings,
        query: CalendarQuery,
    ) -> CalendarFuture<CalendarDelta>;
    fn create(
        &self,
        settings: CalendarSettings,
        event: NewCalendarEvent,
    ) -> CalendarFuture<CalendarEvent>;
}
