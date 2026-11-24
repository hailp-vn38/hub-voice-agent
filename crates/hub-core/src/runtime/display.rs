use crate::{ConfigStore, Device, DeviceSettings, Io, SettingChange};
use jiff::Zoned;
use std::sync::Arc;

/// Giây người dùng có để giữ độ phân giải mới trước khi tự hoàn tác.
const CONFIRM_SECS: i64 = 10;

struct Trial {
    previous: Option<String>,
    chosen: String,
    deadline: i64,
}

/// Điều khiển màn hình: độ sáng theo giờ và phép thử độ phân giải.
pub(super) struct Display {
    device: Arc<dyn Device>,
    brightness: Option<f32>,
    trial: Option<Trial>,
}

impl Display {
    pub fn new(device: Arc<dyn Device>) -> Self {
        Self {
            device,
            brightness: None,
            trial: None,
        }
    }

    /// Đọc trạng thái thật của thiết bị và áp lại độ phân giải đã lưu.
    pub fn init(&mut self, settings: &DeviceSettings, io: &Io) {
        let resolutions = self.device.resolutions();
        if let Some(saved) = settings.resolution.as_deref()
            && resolutions.iter().any(|mode| mode == saved)
            && self.device.current_resolution().as_deref() != Some(saved)
            && let Err(error) = self.device.set_resolution(saved)
        {
            eprintln!("không áp lại được độ phân giải {saved}: {error}");
        }
        let current = self.device.current_resolution();
        io.ui_tx.send_modify(|ui| {
            ui.display.resolutions = resolutions;
            ui.display.resolution = current;
        });
    }

    /// Đặt độ sáng theo cài đặt và giờ hiện tại; chỉ gọi thiết bị khi giá trị đổi.
    pub fn brightness(&mut self, settings: &DeviceSettings, now: &Zoned) {
        let level = settings.effective_brightness(now);
        if self.brightness != Some(level) {
            self.brightness = Some(level);
            self.device.set_brightness(level);
        }
    }

    pub fn try_resolution(&mut self, mode: String, now: i64, io: &Io) {
        let previous = match &self.trial {
            Some(trial) => trial.previous.clone(),
            None => self.device.current_resolution(),
        };
        if let Err(error) = self.device.set_resolution(&mode) {
            eprintln!("không đổi được độ phân giải {mode}: {error}");
            return;
        }
        io.ui_tx.send_modify(|ui| {
            ui.display.resolution = Some(mode.clone());
            ui.display.resolution_confirm = CONFIRM_SECS as u32;
        });
        self.trial = Some(Trial {
            previous,
            chosen: mode,
            deadline: now + CONFIRM_SECS,
        });
    }

    /// Giữ độ phân giải đang thử và lưu vào config.
    pub fn keep(&mut self, config: &mut ConfigStore, io: &Io) {
        let Some(trial) = self.trial.take() else {
            return;
        };
        if let Err(error) =
            config.update(|c| c.device.apply(SettingChange::Resolution(trial.chosen)))
        {
            eprintln!("không lưu được độ phân giải: {error}");
        }
        let device = config.get().device.clone();
        io.ui_tx.send_modify(|ui| {
            ui.display.resolution_confirm = 0;
            ui.settings = device;
        });
    }

    /// Mỗi giây: đếm ngược, hết giờ thì hoàn tác.
    pub fn tick(&mut self, now: i64, io: &Io) {
        let Some(trial) = &self.trial else {
            return;
        };
        let remaining = (trial.deadline - now).max(0) as u32;
        if remaining > 0 {
            io.ui_tx.send_if_modified(|ui| {
                let changed = ui.display.resolution_confirm != remaining;
                ui.display.resolution_confirm = remaining;
                changed
            });
            return;
        }
        let trial = self.trial.take().expect("trial checked above");
        if let Some(previous) = &trial.previous
            && let Err(error) = self.device.set_resolution(previous)
        {
            eprintln!("không hoàn tác được độ phân giải {previous}: {error}");
        }
        io.ui_tx.send_modify(|ui| {
            ui.display.resolution = trial.previous;
            ui.display.resolution_confirm = 0;
        });
    }
}
