use crate::{App, Nav, SettingsData};
use hub_core::UserInput;
use slint::ComponentHandle;
use tokio::sync::mpsc;

/// Nối callback hợp đồng vào hub-core. Gọi trên luồng UI trước `app.run()`.
pub fn connect_inputs(app: &App, input_tx: mpsc::Sender<UserInput>) {
    // Tạm: hub-core chưa có `view` (#05) nên Nav đổi màn ngay tại UI.
    let weak = app.as_weak();
    app.global::<Nav>().on_navigate(move |s| {
        if let Some(app) = weak.upgrade() {
            app.global::<Nav>().set_screen(s);
        }
    });
    let settings = app.global::<SettingsData>();
    let save_tx = input_tx.clone();
    settings.on_save_voice_url(move |url| {
        if save_tx.try_send(UserInput::SaveVoiceUrl(url.to_string())).is_err() {
            eprintln!("hub-core không nhận được URL");
        }
    });
    settings.on_connect_voice(move || {
        if input_tx.try_send(UserInput::VoiceConnect).is_err() {
            eprintln!("hub-core không nhận được lệnh kết nối");
        }
    });
}
