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
