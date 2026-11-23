use crate::CalendarSettings;
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};

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
pub trait CalendarSource: Send + Sync {
    fn list(
        &self,
        settings: CalendarSettings,
        sync_token: Option<String>,
    ) -> CalendarFuture<CalendarDelta>;
    fn create(
        &self,
        settings: CalendarSettings,
        event: NewCalendarEvent,
    ) -> CalendarFuture<CalendarEvent>;
}
