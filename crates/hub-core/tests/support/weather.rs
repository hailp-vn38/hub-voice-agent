use serde_json::{Value, json};

#[derive(Default)]
pub struct FakeWeather {
    pub calls: std::sync::atomic::AtomicUsize,
    pub home_calls: std::sync::atomic::AtomicUsize,
    pub fail: std::sync::atomic::AtomicBool,
    pub weather_code: std::sync::atomic::AtomicI64,
}

pub fn open_meteo_fixture() -> Value {
    let forecast =
        serde_json::from_str(include_str!("../fixtures/open_meteo_forecast.json")).unwrap();
    let air = serde_json::from_str(include_str!("../fixtures/open_meteo_air.json")).unwrap();
    hub_core::OpenMeteo::parse_forecast(forecast, air).unwrap()
}

pub fn sample_forecast(location: String, temperature: i64, weather_code: i64) -> Value {
    let dates: Vec<_> = (0..7)
        .map(|offset| format!("2026-11-{:02}", 23 + offset))
        .collect();
    let times: Vec<_> = (0..7)
        .flat_map(|day| (0..24).map(move |hour| format!("2026-11-{:02}T{hour:02}:00", 23 + day)))
        .collect();
    let temperatures: Vec<_> = (0..7)
        .flat_map(|day| {
            (0..24).map(move |hour| {
                if day == 1 && hour == 14 {
                    35.4
                } else {
                    20.0 + day as f64 + hour as f64 / 10.0
                }
            })
        })
        .collect();
    let daily_highs: Vec<_> = (0..7).map(|day| 29.0 + day as f64).collect();
    let daily_lows: Vec<_> = (0..7).map(|day| 19.0 + day as f64).collect();
    let daily_codes = vec![weather_code; 7];
    let sunrises: Vec<_> = dates.iter().map(|date| format!("{date}T06:00")).collect();
    let sunsets: Vec<_> = dates.iter().map(|date| format!("{date}T17:30")).collect();

    json!({
        "location": location,
        "timezone": "Asia/Ho_Chi_Minh",
        "current": {
            "time": "2026-11-23T12:00",
            "temperature_2m": temperature,
            "apparent_temperature": 31.2,
            "relative_humidity_2m": 78,
            "weather_code": weather_code,
            "wind_speed_10m": 13.3,
            "uv_index": 4.1
        },
        "hourly": {
            "time": times,
            "temperature_2m": temperatures,
            "precipitation_probability": vec![35; 168],
            "weather_code": vec![weather_code; 168],
            "wind_speed_10m": vec![13.3; 168],
            "uv_index": vec![4.1; 168]
        },
        "daily": {
            "time": dates,
            "temperature_2m_max": daily_highs,
            "temperature_2m_min": daily_lows,
            "weather_code": daily_codes,
            "sunrise": sunrises,
            "sunset": sunsets,
            "uv_index_max": vec![7.2; 7]
        },
        "air_quality": {"current": {"us_aqi": 52, "pm2_5": 19.0}}
    })
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
        let weather_code = self.weather_code.load(std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move {
            if fail {
                anyhow::bail!("offline");
            }
            let (name, temperature) = match location {
                hub_core::WeatherLocation::City(name) => (name, 28),
                _ => ("Home".into(), 20),
            };
            Ok(sample_forecast(name, temperature, weather_code))
        })
    }
}
