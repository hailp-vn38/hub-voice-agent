use hub_core::ConfigStore;

#[test]
fn voice_identity_and_url_survive_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = ConfigStore::open(dir.path()).unwrap();
    let original = config.get().clone();
    config
        .update(|c| c.voice.ota_url = Some("http://localhost/voice/ota".into()))
        .unwrap();
    let reopened = ConfigStore::open(dir.path()).unwrap();
    assert_eq!(reopened.get().voice.device_id, original.voice.device_id);
    assert_eq!(reopened.get().voice.client_id, original.voice.client_id);
    assert_eq!(
        reopened.get().voice.ota_url.as_deref(),
        Some("http://localhost/voice/ota")
    );
}

#[test]
fn saving_preserves_external_fields_and_restricts_file_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = ConfigStore::open(dir.path()).unwrap();
    let path = dir.path().join("config.toml");
    let mut doc: toml::Table = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    doc.insert("weather".into(), toml::Value::String("hanoi".into()));
    doc["voice"]
        .as_table_mut()
        .unwrap()
        .insert("other".into(), toml::Value::String("keep".into()));
    std::fs::write(&path, toml::to_string(&doc).unwrap()).unwrap();
    store
        .update(|c| c.voice.ota_url = Some("https://localhost/ota".into()))
        .unwrap();
    let saved: toml::Table = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(saved["weather"].as_str(), Some("hanoi"));
    assert_eq!(saved["voice"]["other"].as_str(), Some("keep"));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn device_settings_default_for_old_files_and_survive_restart() {
    use hub_core::{DeviceSettings, SettingChange, TextSize};
    let dir = tempfile::tempdir().unwrap();
    let mut store = ConfigStore::open(dir.path()).unwrap();
    assert_eq!(store.get().device, DeviceSettings::default());
    store
        .update(|c| c.device.apply(SettingChange::TextSize(TextSize::Large)))
        .unwrap();
    store
        .update(|c| c.device.apply(SettingChange::Volume(0.25)))
        .unwrap();
    let reopened = ConfigStore::open(dir.path()).unwrap();
    assert_eq!(reopened.get().device.text_size, TextSize::Large);
    assert_eq!(reopened.get().device.volume, 0.25);
}
