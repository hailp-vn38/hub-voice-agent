use jiff::Zoned;

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
    pub stale: bool,
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
