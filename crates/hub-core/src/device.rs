/// Thao tác nguồn: khởi động lại app, khởi động lại máy, tắt máy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
    RestartApp,
    Reboot,
    Shutdown,
}

/// Phần cứng màn hình của thiết bị. Chạy đồng bộ: ghi sysfs hoặc gọi lệnh ngắn.
pub trait Device: Send + Sync {
    fn set_brightness(&self, level: f32);
    /// Các chế độ màn hình có thể chọn; rỗng nếu không đổi được.
    fn resolutions(&self) -> Vec<String>;
    fn current_resolution(&self) -> Option<String>;
    fn set_resolution(&self, mode: &str) -> anyhow::Result<()>;
    /// Mặc định không hỗ trợ; bản thật chạy lệnh hệ thống.
    fn power(&self, _action: PowerAction) -> anyhow::Result<()> {
        anyhow::bail!("thiết bị không hỗ trợ thao tác nguồn")
    }
}

/// Thiết bị không điều khiển được gì (desktop, test).
pub struct NoDevice;

impl Device for NoDevice {
    fn set_brightness(&self, _: f32) {}
    fn resolutions(&self) -> Vec<String> {
        Vec::new()
    }
    fn current_resolution(&self) -> Option<String> {
        None
    }
    fn set_resolution(&self, _: &str) -> anyhow::Result<()> {
        anyhow::bail!("thiết bị không đổi được độ phân giải")
    }
}
