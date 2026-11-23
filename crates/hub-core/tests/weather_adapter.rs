use hub_core::OpenMeteo;

#[test]
fn parses_open_meteo_forecast_and_air_quality() {
    let forecast = serde_json::from_str(include_str!("fixtures/open_meteo_forecast.json")).unwrap();
    let air = serde_json::from_str(include_str!("fixtures/open_meteo_air.json")).unwrap();
    let data = OpenMeteo::parse_forecast(forecast, air).unwrap();
    assert_eq!(data["current"]["temperature_2m"], 28.9);
    assert_eq!(data["air_quality"]["current"]["us_aqi"], 52);
}
