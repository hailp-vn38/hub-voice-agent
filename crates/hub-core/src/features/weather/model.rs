use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherForecast {
    pub current: WeatherCurrent,
    pub hours: Vec<WeatherHour>,
    pub days: Vec<WeatherDay>,
    pub air_quality: Option<AirQuality>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherCurrent {
    pub time: String,
    pub temperature: Option<f32>,
    pub apparent_temperature: Option<f32>,
    pub relative_humidity: Option<f32>,
    pub weather_code: Option<i64>,
    pub wind_speed: Option<f32>,
    pub uv_index: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherHour {
    pub time: String,
    pub temperature: Option<f32>,
    pub precipitation_probability: Option<f32>,
    pub weather_code: Option<i64>,
    pub wind_speed: Option<f32>,
    pub uv_index: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherDay {
    pub date: String,
    pub temperature_max: Option<f32>,
    pub temperature_min: Option<f32>,
    pub weather_code: Option<i64>,
    pub sunrise: Option<String>,
    pub sunset: Option<String>,
    pub uv_index_max: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AirQuality {
    pub us_aqi: Option<f32>,
    pub pm2_5: Option<f32>,
}

impl WeatherForecast {
    pub(super) fn from_open_meteo(data: &Value) -> Self {
        let current = &data["current"];
        let current = WeatherCurrent {
            time: string(current, "time").unwrap_or_default(),
            temperature: number(current, "temperature_2m"),
            apparent_temperature: number(current, "apparent_temperature"),
            relative_humidity: number(current, "relative_humidity_2m"),
            weather_code: integer(current, "weather_code"),
            wind_speed: number(current, "wind_speed_10m"),
            uv_index: number(current, "uv_index"),
        };
        let hours = rows(&data["hourly"]["time"])
            .iter()
            .enumerate()
            .filter_map(|(index, time)| {
                Some(WeatherHour {
                    time: time.as_str()?.to_owned(),
                    temperature: number_at(&data["hourly"]["temperature_2m"], index),
                    precipitation_probability: number_at(
                        &data["hourly"]["precipitation_probability"],
                        index,
                    ),
                    weather_code: integer_at(&data["hourly"]["weather_code"], index),
                    wind_speed: number_at(&data["hourly"]["wind_speed_10m"], index),
                    uv_index: number_at(&data["hourly"]["uv_index"], index),
                })
            })
            .collect();
        let days = rows(&data["daily"]["time"])
            .iter()
            .enumerate()
            .filter_map(|(index, date)| {
                Some(WeatherDay {
                    date: date.as_str()?.to_owned(),
                    temperature_max: number_at(&data["daily"]["temperature_2m_max"], index),
                    temperature_min: number_at(&data["daily"]["temperature_2m_min"], index),
                    weather_code: integer_at(&data["daily"]["weather_code"], index),
                    sunrise: string_at(&data["daily"]["sunrise"], index),
                    sunset: string_at(&data["daily"]["sunset"], index),
                    uv_index_max: number_at(&data["daily"]["uv_index_max"], index),
                })
            })
            .collect();
        let air_quality = data["air_quality"]["current"]
            .is_object()
            .then(|| AirQuality {
                us_aqi: number(&data["air_quality"]["current"], "us_aqi"),
                pm2_5: number(&data["air_quality"]["current"], "pm2_5"),
            });

        Self {
            current,
            hours,
            days,
            air_quality,
        }
    }
}

fn rows(value: &Value) -> &[Value] {
    value.as_array().map(Vec::as_slice).unwrap_or_default()
}

fn number(value: &Value, key: &str) -> Option<f32> {
    value[key].as_f64().map(|number| number as f32)
}

fn integer(value: &Value, key: &str) -> Option<i64> {
    value[key].as_i64()
}

fn number_at(value: &Value, index: usize) -> Option<f32> {
    rows(value).get(index)?.as_f64().map(|number| number as f32)
}

fn integer_at(value: &Value, index: usize) -> Option<i64> {
    rows(value).get(index)?.as_i64()
}

fn string(value: &Value, key: &str) -> Option<String> {
    value[key].as_str().map(str::to_owned)
}

fn string_at(value: &Value, index: usize) -> Option<String> {
    rows(value).get(index)?.as_str().map(str::to_owned)
}

pub(crate) fn condition(code: Option<i64>) -> &'static str {
    match code {
        Some(0) => "Trời quang",
        Some(1) => "Ít mây",
        Some(2) => "Có mây rải rác",
        Some(3) => "Nhiều mây",
        Some(45 | 48) => "Sương mù",
        Some(51..=57) => "Mưa phùn",
        Some(61..=67) => "Mưa",
        Some(71..=77 | 85..=86) => "Tuyết",
        Some(80..=82) => "Mưa rào",
        Some(95..=99) => "Giông",
        _ => "Không rõ",
    }
}
