use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct StoredVoice {
    device_id: String,
    client_id: String,
    ota_url: Option<String>,
}

pub struct VoiceSettings {
    path: PathBuf,
    voice: StoredVoice,
}

impl VoiceSettings {
    pub fn open(directory: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(directory)?;
        let path = directory.join("config.toml");
        let mut document = read_document(&path)?;
        let voice = match document.remove("voice") {
            Some(value) => value.try_into().context("invalid voice config")?,
            None => StoredVoice {
                device_id: Uuid::new_v4().to_string(),
                client_id: Uuid::new_v4().to_string(),
                ota_url: None,
            },
        };
        let settings = Self { path, voice };
        if let Some(url) = &settings.voice.ota_url {
            parse_url(url)?;
        }
        settings.persist()?;
        Ok(settings)
    }

    pub fn device_id(&self) -> &str {
        &self.voice.device_id
    }
    pub fn client_id(&self) -> &str {
        &self.voice.client_id
    }
    pub fn ota_url(&self) -> Option<Url> {
        self.voice
            .ota_url
            .as_deref()
            .and_then(|url| parse_url(url).ok())
    }

    pub fn save_url(&mut self, input: &str) -> anyhow::Result<Url> {
        let url = parse_url(input)?;
        let previous = self.voice.ota_url.replace(url.to_string());
        if let Err(error) = self.persist() {
            self.voice.ota_url = previous;
            return Err(error);
        }
        Ok(url)
    }

    fn persist(&self) -> anyhow::Result<()> {
        let mut document = read_document(&self.path)?;
        let voice = document
            .entry("voice")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .context("invalid voice config")?;
        let values = toml::Value::try_from(&self.voice)?;
        voice.extend(
            values
                .as_table()
                .expect("voice serializes as a table")
                .clone(),
        );
        let contents = toml::to_string(&document)?;
        let temporary = self.path.with_extension(format!("{}.tmp", Uuid::new_v4()));
        let result = (|| -> anyhow::Result<()> {
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temporary)?;
            file.write_all(contents.as_bytes())?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}

fn read_document(path: &Path) -> anyhow::Result<toml::Table> {
    match fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).context("invalid config.toml"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(toml::Table::new()),
        Err(error) => Err(error.into()),
    }
}

fn parse_url(input: &str) -> anyhow::Result<Url> {
    let url = Url::parse(input.trim()).context("URL không hợp lệ")?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        bail!("URL phải dùng HTTP hoặc HTTPS và có tên máy chủ");
    }
    Ok(url)
}
