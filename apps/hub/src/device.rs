use std::{fs, path::PathBuf};

/// Màn hình của máy chạy hub: độ sáng qua backlight sysfs nếu có (máy kiosk Linux); không có thì bỏ qua.
/// Độ phân giải chưa hỗ trợ: không liệt kê mode nào.
pub struct HostDevice {
    backlight: Option<PathBuf>,
}

impl HostDevice {
    pub fn new() -> Self {
        let backlight = fs::read_dir("/sys/class/backlight")
            .ok()
            .and_then(|mut dir| dir.find_map(Result::ok))
            .map(|entry| entry.path());
        Self { backlight }
    }
}

impl hub_core::Device for HostDevice {
    fn set_brightness(&self, level: f32) {
        let Some(dir) = &self.backlight else {
            return;
        };
        let Some(max) = fs::read_to_string(dir.join("max_brightness"))
            .ok()
            .and_then(|text| text.trim().parse::<f32>().ok())
        else {
            return;
        };
        let value = (level.clamp(0.0, 1.0) * max).round() as u32;
        if let Err(error) = fs::write(dir.join("brightness"), value.to_string()) {
            eprintln!("không đặt được độ sáng: {error}");
        }
    }
    fn resolutions(&self) -> Vec<String> {
        Vec::new()
    }
    fn current_resolution(&self) -> Option<String> {
        None
    }
    fn set_resolution(&self, _: &str) -> anyhow::Result<()> {
        anyhow::bail!("chưa hỗ trợ đổi độ phân giải")
    }
}
