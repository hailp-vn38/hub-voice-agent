use hub_core::OpenMeteo;

#[test]
fn parses_open_meteo_forecast_and_air_quality() {
    let forecast = serde_json::from_str(include_str!("fixtures/open_meteo_forecast.json")).unwrap();
    let air = serde_json::from_str(include_str!("fixtures/open_meteo_air.json")).unwrap();
    let data = OpenMeteo::parse_forecast(forecast, air).unwrap();
    assert_eq!(data["current"]["temperature_2m"], 28.9);
    assert_eq!(data["air_quality"]["current"]["us_aqi"], 52);
}

#[test]
fn parses_geocoding_places() {
    let geo = serde_json::json!({"results": [
        {"name": "Hà Nội", "admin1": "Hà Nội", "country": "Việt Nam", "latitude": 21.02, "longitude": 105.84},
        {"name": "Bad"}
    ]});
    let places = OpenMeteo::parse_places(&geo);
    assert_eq!(places.len(), 1);
    assert_eq!(places[0].name, "Hà Nội, Hà Nội, Việt Nam");
    assert_eq!(places[0].latitude, 21.02);
}
