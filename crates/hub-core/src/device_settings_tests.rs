use super::*;

#[test]
fn levels_are_clamped_to_the_unit_range() {
    let mut settings = DeviceSettings::default();
    settings.apply(SettingChange::Brightness(1.7));
    settings.apply(SettingChange::Volume(-0.2));
    assert_eq!((settings.brightness, settings.volume), (1.0, 0.0));
    settings.apply(SettingChange::Volume(f32::NAN));
    assert_eq!(settings.volume, 0.0);
}

#[test]
fn each_change_updates_only_its_field() {
    let mut settings = DeviceSettings::default();
    settings.apply(SettingChange::TextSize(TextSize::Large));
    settings.apply(SettingChange::MicMuted(true));
    let expected = DeviceSettings {
        text_size: TextSize::Large,
        mic_muted: true,
        ..DeviceSettings::default()
    };
    assert_eq!(settings, expected);
}

#[test]
fn partial_toml_falls_back_to_defaults() {
    let settings: DeviceSettings = toml::from_str("text_size = \"small\"").unwrap();
    assert_eq!(settings.text_size, TextSize::Small);
    assert_eq!(settings.volume, DeviceSettings::default().volume);
}

fn at(time: &str) -> jiff::Zoned {
    format!("2026-11-22T{time}+07:00[Asia/Ho_Chi_Minh]")
        .parse()
        .unwrap()
}

#[test]
fn night_range_wraps_past_midnight() {
    let settings = DeviceSettings::default(); // 22:00–06:00
    assert!(settings.in_night_range(&at("23:30:00")));
    assert!(settings.in_night_range(&at("05:59:00")));
    assert!(!settings.in_night_range(&at("06:00:00")));
    assert!(!settings.in_night_range(&at("12:00:00")));
}

#[test]
fn night_range_inside_one_day_and_bad_format() {
    let mut settings = DeviceSettings::default();
    settings.apply(SettingChange::AmbientNightRange("13:00–15:00".into()));
    assert!(settings.in_night_range(&at("14:00:00")));
    assert!(!settings.in_night_range(&at("15:00:00")));
    settings.apply(SettingChange::AmbientNightRange("tối".into()));
    assert!(!settings.in_night_range(&at("23:00:00")));
}

#[test]
fn night_flag_needs_the_toggle() {
    let mut settings = DeviceSettings::default();
    assert!(settings.night_now(&at("23:00:00")));
    settings.apply(SettingChange::AmbientNight(false));
    assert!(!settings.night_now(&at("23:00:00")));
}

#[test]
fn auto_brightness_dims_only_at_night() {
    let mut settings = DeviceSettings::default();
    settings.apply(SettingChange::Brightness(1.0));
    assert_eq!(settings.effective_brightness(&at("23:00:00")), 1.0);
    settings.apply(SettingChange::AutoBrightness(true));
    assert_eq!(settings.effective_brightness(&at("23:00:00")), 0.3);
    assert_eq!(settings.effective_brightness(&at("12:00:00")), 1.0);
}

#[test]
fn ambient_delay_in_seconds() {
    assert_eq!(AmbientAfter::Minute1.seconds(), Some(60));
    assert_eq!(AmbientAfter::Never.seconds(), None);
}

#[test]
fn center_entities_are_limited_and_deselectable() {
    let mut settings = DeviceSettings::default();
    for id in ["a", "b", "c", "d", "e"] {
        settings.apply(SettingChange::CenterEntity {
            id: id.into(),
            selected: true,
        });
    }
    assert_eq!(settings.center_entities, ["a", "b", "c", "d"]);
    settings.apply(SettingChange::CenterEntity {
        id: "b".into(),
        selected: false,
    });
    settings.apply(SettingChange::CenterEntity {
        id: "e".into(),
        selected: true,
    });
    assert_eq!(settings.center_entities, ["a", "c", "d", "e"]);
}
