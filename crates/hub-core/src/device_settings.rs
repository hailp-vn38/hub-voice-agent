use crate::Sensitivity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AmbientAfter {
    Minute1,
    #[default]
    Minutes5,
    Minutes10,
    Never,
}

/// Cài đặt thiết bị do người dùng chỉnh ở màn Setting; lưu trong mục `[device]` của config.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DeviceSettings {
    pub brightness: f32,
    pub auto_brightness: bool,
    pub resolution: Option<String>,
    pub text_size: TextSize,
    pub ambient_after: AmbientAfter,
    pub ambient_night: bool,
    pub ambient_night_range: String,
    pub volume: f32,
    pub ui_sounds: bool,
    pub timer_sound: String,
    pub reminder_sound: String,
    pub wake_word: bool,
    pub sensitivity: Sensitivity,
    pub mic_muted: bool,
    /// Id thiết bị Home Assistant hiện ở màn trung tâm; rỗng = bốn thiết bị đầu tiên.
    pub center_entities: Vec<String>,
    pub timezone: Option<String>,
}

impl Default for DeviceSettings {
    fn default() -> Self {
        Self {
            brightness: 0.8,
            auto_brightness: false,
            resolution: None,
            text_size: TextSize::default(),
            ambient_after: AmbientAfter::default(),
            ambient_night: true,
            ambient_night_range: "22:00–06:00".into(),
            volume: 1.0,
            ui_sounds: true,
            timer_sound: "Reminder".into(),
            reminder_sound: "Reminder".into(),
            wake_word: true,
            sensitivity: Sensitivity::default(),
            mic_muted: false,
            center_entities: Vec::new(),
            timezone: None,
        }
    }
}

/// Một thay đổi người dùng thực hiện trên màn Setting.
#[derive(Clone, Debug, PartialEq)]
pub enum SettingChange {
    Brightness(f32),
    AutoBrightness(bool),
    Resolution(String),
    TextSize(TextSize),
    AmbientAfter(AmbientAfter),
    AmbientNight(bool),
    AmbientNightRange(String),
    Volume(f32),
    UiSounds(bool),
    TimerSound(String),
    ReminderSound(String),
    WakeWord(bool),
    Sensitivity(Sensitivity),
    MicMuted(bool),
    Timezone(String),
    CenterEntity { id: String, selected: bool },
}

impl AmbientAfter {
    /// Số giây không chạm trước khi vào Ambient; `None` = không bao giờ.
    pub fn seconds(self) -> Option<i64> {
        match self {
            Self::Minute1 => Some(60),
            Self::Minutes5 => Some(300),
            Self::Minutes10 => Some(600),
            Self::Never => None,
        }
    }
}

/// Số thiết bị tối đa hiện ở màn trung tâm.
pub const MAX_CENTER_ENTITIES: usize = 4;

/// Độ sáng ban đêm khi "Theo giờ", tính theo độ sáng đã chọn.
const NIGHT_DIM: f32 = 0.3;

impl DeviceSettings {
    /// `now` có nằm trong khung giờ ban đêm đã chọn không. Khung sai định dạng coi như không có.
    pub fn in_night_range(&self, now: &jiff::Zoned) -> bool {
        let Some((start, end)) = night_range(&self.ambient_night_range) else {
            return false;
        };
        let minute = i32::from(now.hour()) * 60 + i32::from(now.minute());
        if start <= end {
            (start..end).contains(&minute)
        } else {
            minute >= start || minute < end
        }
    }

    /// Ambient ban đêm: bật thì tối đi trong khung giờ, tắt thì không bao giờ.
    pub fn night_now(&self, now: &jiff::Zoned) -> bool {
        self.ambient_night && self.in_night_range(now)
    }

    /// Độ sáng thực tế: "Theo giờ" giảm độ sáng trong khung ban đêm.
    pub fn effective_brightness(&self, now: &jiff::Zoned) -> f32 {
        if self.auto_brightness && self.in_night_range(now) {
            self.brightness * NIGHT_DIM
        } else {
            self.brightness
        }
    }

    pub fn apply(&mut self, change: SettingChange) {
        match change {
            SettingChange::Brightness(v) => self.brightness = unit(v),
            SettingChange::AutoBrightness(v) => self.auto_brightness = v,
            SettingChange::Resolution(v) => self.resolution = Some(v),
            SettingChange::TextSize(v) => self.text_size = v,
            SettingChange::AmbientAfter(v) => self.ambient_after = v,
            SettingChange::AmbientNight(v) => self.ambient_night = v,
            SettingChange::AmbientNightRange(v) => self.ambient_night_range = v,
            SettingChange::Volume(v) => self.volume = unit(v),
            SettingChange::UiSounds(v) => self.ui_sounds = v,
            SettingChange::TimerSound(v) => self.timer_sound = v,
            SettingChange::ReminderSound(v) => self.reminder_sound = v,
            SettingChange::WakeWord(v) => self.wake_word = v,
            SettingChange::Sensitivity(v) => self.sensitivity = v,
            SettingChange::MicMuted(v) => self.mic_muted = v,
            SettingChange::Timezone(v) => self.timezone = Some(v),
            SettingChange::CenterEntity { id, selected } => {
                self.center_entities.retain(|chosen| *chosen != id);
                if selected && self.center_entities.len() < MAX_CENTER_ENTITIES {
                    self.center_entities.push(id);
                }
            }
        }
    }
}

/// "HH:MM–HH:MM" → số phút từ nửa đêm của hai đầu.
fn night_range(range: &str) -> Option<(i32, i32)> {
    let (start, end) = range.split_once(['–', '-'])?;
    let minutes = |text: &str| {
        let (hour, minute) = text.trim().split_once(':')?;
        let (hour, minute) = (hour.parse::<i32>().ok()?, minute.parse::<i32>().ok()?);
        ((0..24).contains(&hour) && (0..60).contains(&minute)).then_some(hour * 60 + minute)
    };
    Some((minutes(start)?, minutes(end)?))
}

/// Đưa về [0, 1]; NaN thành 0.
fn unit(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "device_settings_tests.rs"]
mod tests;
