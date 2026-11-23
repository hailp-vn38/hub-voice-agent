#[allow(dead_code)]
mod support;

use hub_core::{Focus, UserInput, WeatherBackground, WeatherLocation, WeatherSource, present};
use serde_json::Value;
use std::{future::Future, pin::Pin, sync::Arc, sync::atomic::Ordering, time::Duration};

struct OpenMeteoFixture;

impl WeatherSource for OpenMeteoFixture {
    fn forecast(
        &self,
        _location: WeatherLocation,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Value>> + Send + '_>> {
        let fixture = support::open_meteo_fixture();
        Box::pin(async move { Ok(fixture) })
    }
}

#[tokio::test(start_paused = true)]
async fn run_parses_the_open_meteo_sample_before_presenting_it() {
    let start = "2026-11-23T12:15:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let hub = support::Hub::start_with_weather(start, Arc::new(OpenMeteoFixture));

    for _ in 0..20 {
        if hub.ui_rx.borrow().weather.forecast.is_some() {
            break;
        }
        tokio::task::yield_now().await;
    }

    let state = hub.ui_rx.borrow();
    let forecast = state.weather.forecast.as_ref().unwrap();
    assert_eq!(forecast.current.temperature, Some(28.9));
    assert_eq!(forecast.air_quality.as_ref().unwrap().us_aqi, Some(52.0));
    let weather = present(&state).weather;
    assert_eq!(weather.now.temp, "29°");
    assert_eq!(weather.now.aqi, "52");
    assert_eq!(weather.background, WeatherBackground::Cloudy);
    assert_eq!(weather.hours.len(), 24);

    drop(state);
    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}

#[tokio::test(start_paused = true)]
async fn run_parses_weather_and_present_tracks_the_focused_day_and_stale_state() {
    let start = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let source = Arc::new(support::FakeWeather::default());
    source.weather_code.store(61, Ordering::SeqCst);
    let hub = support::Hub::start_with_weather(start, source.clone());

    for _ in 0..20 {
        if hub.ui_rx.borrow().weather.forecast.is_some() {
            break;
        }
        tokio::task::yield_now().await;
    }

    let weather = present(&hub.ui_rx.borrow()).weather;
    assert_eq!(weather.location, "Home");
    assert!(!weather.preview);
    assert_eq!(weather.updated, "12:00");
    assert!(!weather.stale);
    assert_eq!(weather.background, WeatherBackground::Rain);
    assert_eq!(weather.selected_day, "2026-11-23");
    assert!(weather.focus.is_empty());
    assert_eq!(weather.now.temp, "20°");
    assert_eq!(weather.now.feels, "31°");
    assert_eq!(weather.now.humidity, "78%");
    assert_eq!(weather.now.wind, "13 km/h");
    assert_eq!(weather.now.aqi, "52");
    assert_eq!(weather.now.aqi_level, "Trung bình");
    assert_eq!(weather.now.aqi_color, "#FFFF00");
    assert_eq!(weather.now.sunrise, "06:00");
    assert_eq!(weather.now.sunset, "17:30");
    assert!(weather.is_day);
    assert_eq!(weather.hours.len(), 24);
    assert_eq!(
        weather.hours.iter().find(|hour| hour.peak).unwrap().key,
        "2026-11-23T23:00"
    );
    assert_eq!(weather.days.len(), 7);
    assert_eq!(weather.days[1].label, "Mai");
    assert!(weather.days[1].is_tomorrow);
    assert_eq!(weather.week_min, 19.0);
    assert_eq!(weather.week_max, 35.0);

    hub.input_tx
        .send(UserInput::Focus(Some(Focus::Weather(
            "2026-11-24T14:00".into(),
        ))))
        .await
        .unwrap();
    tokio::task::yield_now().await;

    let weather = present(&hub.ui_rx.borrow()).weather;
    assert_eq!(weather.selected_day, "2026-11-24");
    assert_eq!(weather.hours.len(), 24);
    assert_eq!(
        weather.hours.iter().find(|hour| hour.peak).unwrap().key,
        "2026-11-24T14:00"
    );
    assert_eq!(weather.focus, "2026-11-24T14:00");
    assert_eq!(weather.focus_serial, 0);

    source.fail.store(true, Ordering::SeqCst);
    tokio::time::advance(Duration::from_secs(1801)).await;
    for _ in 0..20 {
        if hub.ui_rx.borrow().weather.stale {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(present(&hub.ui_rx.borrow()).weather.stale);

    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}
