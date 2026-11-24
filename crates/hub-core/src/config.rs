use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tokio::sync::watch;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoiceSettings {
    pub device_id: String,
    pub client_id: String,
    pub ota_url: Option<String>,
    #[serde(default = "default_reminder_speech")]
    pub reminder_speech: bool,
}
fn default_reminder_speech() -> bool {
    true
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalendarSettings {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct HomeSettings {
    pub url: String,
    pub token: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeatherSettings {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub name: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub voice: VoiceSettings,
    pub calendar: CalendarSettings,
    pub home: HomeSettings,
    pub weather: WeatherSettings,
}

pub struct ConfigStore {
    path: PathBuf,
    value: Config,
    tx: watch::Sender<Config>,
}
impl ConfigStore {
    pub fn open(dir: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(dir)?;
        let path = dir.join("config.toml");
        let document = read(&path)?;
        let voice = match document.get("voice") {
            Some(v) => v.clone().try_into().context("invalid voice config")?,
            None => VoiceSettings {
                device_id: Uuid::new_v4().to_string(),
                client_id: Uuid::new_v4().to_string(),
                ota_url: None,
                reminder_speech: true,
            },
        };
        let value = Config {
            voice,
            calendar: document
                .get("calendar")
                .cloned()
                .map(toml::Value::try_into)
                .transpose()?
                .unwrap_or_default(),
            home: document
                .get("home")
                .cloned()
                .map(toml::Value::try_into)
                .transpose()?
                .unwrap_or_default(),
            weather: document
                .get("weather")
                .filter(|v| v.is_table())
                .cloned()
                .map(toml::Value::try_into)
                .transpose()?
                .unwrap_or_default(),
        };
        if let Some(url) = &value.voice.ota_url {
            crate::voice_settings::parse_url(url)?;
        }
        let (tx, _) = watch::channel(value.clone());
        let store = Self { path, value, tx };
        store.persist(&store.value)?;
        Ok(store)
    }
    pub fn get(&self) -> &Config {
        &self.value
    }
    pub fn watch(&self) -> watch::Receiver<Config> {
        self.tx.subscribe()
    }
    pub fn update(&mut self, f: impl FnOnce(&mut Config)) -> anyhow::Result<()> {
        let mut next = self.value.clone();
        f(&mut next);
        if let Some(url) = &next.voice.ota_url {
            crate::voice_settings::parse_url(url)?;
        }
        self.persist(&next)?;
        self.value = next.clone();
        self.tx.send_replace(next);
        Ok(())
    }
    fn persist(&self, value: &Config) -> anyhow::Result<()> {
        let mut doc = read(&self.path)?;
        let voice = doc
            .entry("voice")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .context("invalid voice config")?;
        // Remove a previously saved URL when clearing it, while retaining unknown keys.
        voice.remove("ota_url");
        voice.extend(
            toml::Value::try_from(&value.voice)?
                .as_table()
                .unwrap()
                .clone(),
        );
        for (key, settings) in [
            ("calendar", toml::Value::try_from(&value.calendar)?),
            ("home", toml::Value::try_from(&value.home)?),
            ("weather", toml::Value::try_from(&value.weather)?),
        ] {
            if let Some(table) = doc.get_mut(key).and_then(toml::Value::as_table_mut) {
                table.extend(settings.as_table().unwrap().clone());
            } else if !doc.contains_key(key)
                || (key == "weather" && value.weather.latitude.is_some())
            {
                doc.insert(key.into(), settings);
            }
        }
        let contents = toml::to_string(&doc)?;
        let tmp = self.path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        let result = (|| -> anyhow::Result<()> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&tmp)?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            fs::rename(&tmp, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(tmp);
        }
        result
    }
}
fn read(path: &Path) -> anyhow::Result<toml::Table> {
    match fs::read_to_string(path) {
        Ok(s) => toml::from_str(&s).context("invalid config.toml"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(toml::Table::new()),
        Err(e) => Err(e.into()),
    }
}
