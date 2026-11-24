use serde_json::Value;

/// One geocoding match for a place name.
#[derive(Clone, Debug, PartialEq)]
pub struct GeoPlace {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Clone, Default)]
pub struct OpenMeteo {
    client: reqwest::Client,
}

impl OpenMeteo {
    /// Looks up places by name (up to 5) to pick the home location from.
    pub async fn search(&self, query: &str) -> anyhow::Result<Vec<GeoPlace>> {
        let geo: Value = self
            .client
            .get("https://geocoding-api.open-meteo.com/v1/search")
            .query(&[("name", query), ("count", "5"), ("language", "vi")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(Self::parse_places(&geo))
    }
    pub fn parse_places(geo: &Value) -> Vec<GeoPlace> {
        geo["results"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| {
                let parts: Vec<&str> = ["name", "admin1", "country"]
                    .iter()
                    .filter_map(|k| r[*k].as_str())
                    .collect();
                Some(GeoPlace {
                    name: parts.join(", "),
                    latitude: r["latitude"].as_f64()?,
                    longitude: r["longitude"].as_f64()?,
                })
            })
            .collect()
    }
    pub fn parse_forecast(mut forecast: Value, air: Value) -> anyhow::Result<Value> {
        anyhow::ensure!(
            forecast["current"]["temperature_2m"].is_number(),
            "missing current temperature"
        );
        anyhow::ensure!(air["current"].is_object(), "missing air quality");
        forecast
            .as_object_mut()
            .ok_or_else(|| anyhow::anyhow!("invalid forecast"))?
            .insert("air_quality".into(), air);
        Ok(forecast)
    }
}

impl crate::WeatherSource for OpenMeteo {
    fn forecast(
        &self,
        location: crate::WeatherLocation,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Value>> + Send + '_>>
    {
        Box::pin(async move {
            let (latitude, longitude, name) = match location {
                crate::WeatherLocation::Coordinates {
                    latitude,
                    longitude,
                } => (latitude, longitude, None),
                crate::WeatherLocation::City(name) => {
                    let geo: Value = self
                        .client
                        .get("https://geocoding-api.open-meteo.com/v1/search")
                        .query(&[("name", name.as_str()), ("count", "1"), ("language", "vi")])
                        .send()
                        .await?
                        .error_for_status()?
                        .json()
                        .await?;
                    let result = geo["results"]
                        .as_array()
                        .and_then(|a| a.first())
                        .ok_or_else(|| anyhow::anyhow!("City not found"))?;
                    (
                        result["latitude"]
                            .as_f64()
                            .ok_or_else(|| anyhow::anyhow!("missing latitude"))?,
                        result["longitude"]
                            .as_f64()
                            .ok_or_else(|| anyhow::anyhow!("missing longitude"))?,
                        Some(name),
                    )
                }
            };
            let coords = [
                ("latitude", latitude.to_string()),
                ("longitude", longitude.to_string()),
            ];
            let forecast: Value = self.client.get("https://api.open-meteo.com/v1/forecast").query(&coords).query(&[("current", "temperature_2m,apparent_temperature,relative_humidity_2m,weather_code,wind_speed_10m,uv_index"), ("hourly", "temperature_2m,precipitation_probability,weather_code,wind_speed_10m,uv_index"), ("daily", "temperature_2m_max,temperature_2m_min,weather_code,sunrise,sunset,uv_index_max"), ("timezone", "auto"), ("forecast_days", "7")]).send().await?.error_for_status()?.json().await?;
            let air: Value = self
                .client
                .get("https://air-quality-api.open-meteo.com/v1/air-quality")
                .query(&coords)
                .query(&[("current", "us_aqi,pm2_5")])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let mut data = Self::parse_forecast(forecast, air)?;
            if let Some(name) = name {
                data["location"] = name.into();
            }
            Ok(data)
        })
    }
}
