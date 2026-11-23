use serde_json::{Value, json};

#[derive(Default)]
pub struct FakeWeather {
    pub calls: std::sync::atomic::AtomicUsize,
    pub home_calls: std::sync::atomic::AtomicUsize,
    pub fail: std::sync::atomic::AtomicBool,
}
impl hub_core::WeatherSource for FakeWeather {
    fn forecast(
        &self,
        location: hub_core::WeatherLocation,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Value>> + Send + '_>>
    {
        let counter = match &location {
            hub_core::WeatherLocation::City(_) => &self.calls,
            _ => &self.home_calls,
        };
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let fail = self.fail.load(std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            if fail {
                anyhow::bail!("offline");
            }
            let (name, temperature) = match location {
                hub_core::WeatherLocation::City(name) => (name, 28),
                _ => ("Home".into(), 20),
            };
            Ok(
                json!({"location": name, "current": {"temperature_2m": temperature, "weather_code": 0}}),
            )
        })
    }
}
