use crate::{
    App, AudioSettings, DisplaySettings, HomeEntity, HomeSettings, SettingsData, SystemSettings,
    Theme, VoiceSettingsView,
};
use hub_core::{
    AmbientAfter, DataKind, DeviceSettings, DisplayState, HomeEntity as CoreHomeEntity,
    PowerAction, SOUND_NAMES, Sensitivity, SettingChange, TextSize, UserInput,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{cell::RefCell, rc::Rc};

use super::inputs::Inputs;

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

const TIMEZONES: [&str; 12] = [
    "Asia/Ho_Chi_Minh",
    "Asia/Bangkok",
    "Asia/Singapore",
    "Asia/Shanghai",
    "Asia/Tokyo",
    "Asia/Seoul",
    "Australia/Sydney",
    "Europe/London",
    "Europe/Paris",
    "America/New_York",
    "America/Los_Angeles",
    "UTC",
];

const NIGHT_RANGES: [&str; 3] = ["22:00–06:00", "21:00–07:00", "23:00–05:00"];

const AUTO_BRIGHTNESS: [(&str, bool); 2] = [("Tắt", false), ("Theo giờ", true)];
const TEXT_SIZES: [(&str, TextSize); 3] = [
    ("Nhỏ", TextSize::Small),
    ("Vừa", TextSize::Medium),
    ("Lớn", TextSize::Large),
];
const AMBIENT_AFTER: [(&str, AmbientAfter); 4] = [
    ("1 phút", AmbientAfter::Minute1),
    ("5 phút", AmbientAfter::Minutes5),
    ("10 phút", AmbientAfter::Minutes10),
    ("Không", AmbientAfter::Never),
];
const SENSITIVITY: [(&str, Sensitivity); 3] = [
    ("Thấp", Sensitivity::Low),
    ("Vừa", Sensitivity::Medium),
    ("Cao", Sensitivity::High),
];

fn label<T: PartialEq>(table: &[(&'static str, T)], value: &T) -> &'static str {
    table
        .iter()
        .find(|(_, v)| v == value)
        .map_or("", |(label, _)| label)
}

fn value<T: Clone>(table: &[(&str, T)], label: &str) -> Option<T> {
    table
        .iter()
        .find(|(l, _)| *l == label)
        .map(|(_, v)| v.clone())
}

/// Lựa chọn trên UI (khoá mục, nhãn) → thay đổi có kiểu. `None` nếu khoá hoặc nhãn lạ.
pub(super) fn choice(key: &str, picked: &str) -> Option<SettingChange> {
    let text = || picked.to_string();
    Some(match key {
        "auto-brightness" => SettingChange::AutoBrightness(value(&AUTO_BRIGHTNESS, picked)?),
        "resolution" => SettingChange::Resolution(text()),
        "text-size" => SettingChange::TextSize(value(&TEXT_SIZES, picked)?),
        "ambient-after" => SettingChange::AmbientAfter(value(&AMBIENT_AFTER, picked)?),
        "ambient-night-range" => SettingChange::AmbientNightRange(text()),
        "timer-sound" => SettingChange::TimerSound(text()),
        "reminder-sound" => SettingChange::ReminderSound(text()),
        "sensitivity" => SettingChange::Sensitivity(value(&SENSITIVITY, picked)?),
        "timezone" => SettingChange::Timezone(text()),
        _ => return None,
    })
}

/// Công tắc trên UI → thay đổi có kiểu; khoá `entity:<id>` chọn thiết bị cho màn trung tâm.
pub(super) fn toggle(key: &str, on: bool) -> Option<SettingChange> {
    if let Some(id) = key.strip_prefix("entity:") {
        return Some(SettingChange::CenterEntity {
            id: id.to_string(),
            selected: on,
        });
    }
    Some(match key {
        "ambient-night" => SettingChange::AmbientNight(on),
        "ui-sounds" => SettingChange::UiSounds(on),
        "wake-word" => SettingChange::WakeWord(on),
        "mic-muted" => SettingChange::MicMuted(on),
        _ => return None,
    })
}

fn strings(items: impl IntoIterator<Item = impl Into<SharedString>>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        items.into_iter().map(Into::into).collect::<Vec<_>>(),
    ))
}

fn display(settings: &DeviceSettings, state: &DisplayState) -> DisplaySettings {
    DisplaySettings {
        brightness: settings.brightness,
        auto_brightness: label(&AUTO_BRIGHTNESS, &settings.auto_brightness).into(),
        resolution: state.resolution.as_deref().unwrap_or("").into(),
        resolutions: strings(state.resolutions.iter().map(String::as_str)),
        text_size: label(&TEXT_SIZES, &settings.text_size).into(),
        ambient_after: label(&AMBIENT_AFTER, &settings.ambient_after).into(),
        ambient_night: settings.ambient_night,
        ambient_night_range: settings.ambient_night_range.as_str().into(),
        ambient_night_ranges: strings(NIGHT_RANGES),
    }
}

fn audio(settings: &DeviceSettings) -> AudioSettings {
    AudioSettings {
        volume: settings.volume,
        ui_sounds: settings.ui_sounds,
        timer_sound: settings.timer_sound.as_str().into(),
        reminder_sound: settings.reminder_sound.as_str().into(),
        sounds: strings(SOUND_NAMES),
    }
}

/// Hệ số `Theme.text-scale` cho từng cỡ chữ.
pub(super) fn text_scale(size: TextSize) -> f32 {
    match size {
        TextSize::Small => 0.85,
        TextSize::Medium => 1.0,
        TextSize::Large => 1.25,
    }
}

/// Khoá `system-action(...)` của màn Hệ thống → thao tác nguồn.
pub(super) fn power(key: &str) -> Option<PowerAction> {
    Some(match key {
        "restart-app" => PowerAction::RestartApp,
        "reboot" => PowerAction::Reboot,
        "shutdown" => PowerAction::Shutdown,
        _ => return None,
    })
}

/// Múi giờ chọn được; múi giờ đang dùng được thêm vào đầu nếu chưa có trong danh sách.
fn timezones(current: &str) -> Vec<&str> {
    let mut zones = TIMEZONES.to_vec();
    if !current.is_empty() && !zones.contains(&current) {
        zones.insert(0, current);
    }
    zones
}

/// Khoá `clear(...)` của màn Dữ liệu → loại dữ liệu.
pub(super) fn data_kind(key: &str) -> Option<DataKind> {
    Some(match key {
        "photos" => DataKind::Photos,
        "cache" => DataKind::Cache,
        "timers" => DataKind::Timers,
        "countdowns" => DataKind::Countdowns,
        "reminders" => DataKind::Reminders,
        _ => return None,
    })
}

fn home(settings: &DeviceSettings, entities: &[CoreHomeEntity]) -> HomeSettings {
    let rows = entities
        .iter()
        .map(|entity| HomeEntity {
            id: entity.id.as_str().into(),
            name: entity.name.as_str().into(),
            selected: settings.center_entities.contains(&entity.id),
        })
        .collect::<Vec<_>>();
    HomeSettings {
        entities: ModelRc::new(VecModel::from(rows)),
    }
}

pub(super) fn apply(
    app: &App,
    settings: &DeviceSettings,
    state: &DisplayState,
    entities: &[CoreHomeEntity],
    system_zone: &str,
) {
    app.global::<Theme>()
        .set_text_scale(text_scale(settings.text_size));
    let data = app.global::<SettingsData>();
    data.set_resolution_confirm(state.resolution_confirm as i32);
    data.set_display(display(settings, state));
    data.set_home(home(settings, entities));
    data.set_audio(audio(settings));
    data.set_voice(VoiceSettingsView {
        wake_word: settings.wake_word,
        sensitivity: label(&SENSITIVITY, &settings.sensitivity).into(),
        mic_muted: settings.mic_muted,
        ..data.get_voice()
    });
    let zone = settings.timezone.as_deref().unwrap_or(system_zone);
    data.set_system(SystemSettings {
        version: env!("CARGO_PKG_VERSION").into(),
        timezone: zone.into(),
        timezones: strings(timezones(zone)),
    });
}

pub(super) fn connect_inputs(app: &App, inputs: Rc<RefCell<Inputs>>) {
    let data = app.global::<SettingsData>();
    let keep = inputs.clone();
    let send = move |input: Option<UserInput>| {
        if let Some(input) = input {
            let mut inputs = inputs.borrow_mut();
            inputs.activity();
            inputs.send(input);
        }
    };
    let brightness = send.clone();
    data.on_set_brightness(move |v| {
        brightness(Some(UserInput::Setting(SettingChange::Brightness(v))))
    });
    let volume = send.clone();
    data.on_set_volume(move |v| volume(Some(UserInput::Setting(SettingChange::Volume(v)))));
    let chosen = send.clone();
    data.on_choose(move |key, picked| chosen(choice(&key, &picked).map(UserInput::Setting)));
    let preview = send.clone();
    data.on_preview_sound(move |name| preview(Some(UserInput::PreviewSound(name.to_string()))));
    let toggled = send.clone();
    data.on_toggle(move |key, on| {
        toggled(if key == "read-reminder" {
            Some(UserInput::ReminderSpeech(on))
        } else {
            toggle(&key, on).map(UserInput::Setting)
        })
    });
    let clear = send.clone();
    data.on_clear(move |key| clear(data_kind(&key).map(UserInput::Clear)));
    let system = send.clone();
    data.on_system_action(move |key| system(power(&key).map(UserInput::Power)));
    data.on_keep_resolution(move || {
        let mut input = keep.borrow_mut();
        input.activity();
        input.send(UserInput::KeepResolution);
    });
}
