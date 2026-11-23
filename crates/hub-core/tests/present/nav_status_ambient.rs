use hub_core::{Screen, UiState, VoiceState, WeatherCurrent, WeatherDay, WeatherForecast, present};

#[test]
fn present_maps_nav_status_and_ambient_from_the_snapshot_time() {
    let mut state = UiState::default();
    state.now = "2026-11-23T05:45:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.view.screen = Screen::Calendar;
    state.voice.state = VoiceState::Offline;
    state.weather.forecast = Some(WeatherForecast {
        current: WeatherCurrent {
            temperature: Some(28.9),
            weather_code: Some(61),
            ..WeatherCurrent::default()
        },
        days: vec![WeatherDay {
            date: "2026-11-23".into(),
            sunrise: Some("2026-11-23T06:10".into()),
            sunset: Some("2026-11-23T17:20".into()),
            ..WeatherDay::default()
        }],
        ..WeatherForecast::default()
    });

    let view = present(&state);

    assert_eq!(view.nav, Screen::Calendar);
    assert_eq!(view.status.time, "05:45");
    assert!(view.status.offline);
    assert_eq!(view.ambient.time, "05:45");
    assert_eq!(view.ambient.temp, "29°");
    assert_eq!(view.ambient.icon, "rain");
    assert!(view.ambient.night);
    assert!(!view.weather.is_day);
}
