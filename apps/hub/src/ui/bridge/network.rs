//! IP LAN của hub, để mở Setup web từ máy khác cùng mạng Wi-Fi.
use crate::{App, ConnectionSettings, SettingsData};
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer, Weak};
use std::{
    net::{IpAddr, UdpSocket},
    time::Duration,
};

/// IP của giao diện mạng đi ra ngoài. `connect` trên UDP chỉ chọn route, không gửi gói nào.
fn lan_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then_some(ip)
}

fn setup_url(ip: Option<IpAddr>, port: u16) -> String {
    ip.map(|ip| format!("http://{ip}:{port}"))
        .unwrap_or_default()
}

fn apply(app: &App, ip: Option<IpAddr>, port: u16, qr: Option<SharedPixelBuffer<Rgba8Pixel>>) {
    let settings = app.global::<SettingsData>();
    settings.set_setup_url(setup_url(ip, port).into());
    settings.set_setup_qr(qr.map(Image::from_rgba8).unwrap_or_default());
    settings.set_connection(ConnectionSettings {
        wifi_ip: ip.map(|ip| ip.to_string()).unwrap_or_default().into(),
        setup_url: setup_url(ip, port).into(),
        ..settings.get_connection()
    });
}

/// Kiểm tra lại IP mỗi 10 giây (Wi-Fi có thể kết nối sau khi hub khởi động).
pub fn spawn_network(app: Weak<App>, setup_port: u16) {
    tokio::spawn(async move {
        let mut shown = None;
        loop {
            let ip = lan_ip();
            if shown != Some(ip) {
                shown = Some(ip);
                let app = app.clone();
                let qr = super::qr::render(&setup_url(ip, setup_port));
                if slint::invoke_from_event_loop(move || {
                    if let Some(app) = app.upgrade() {
                        apply(&app, ip, setup_port, qr);
                    }
                })
                .is_err()
                {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    });
}

#[cfg(test)]
#[path = "network_tests.rs"]
mod tests;
