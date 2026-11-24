//! Chọn vị trí nhà cho Open-Meteo: tìm theo tên (geocoding), chọn một kết quả để lưu.
use crate::{App, LocationOption, SettingsData};
use hub_core::{GeoPlace, OpenMeteo, UserInput, WeatherSettings, setup_web::Update};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;
use tokio::{runtime::Handle, sync::mpsc, sync::oneshot};

fn show(app: &App, message: &str, places: Vec<GeoPlace>) {
    let settings = app.global::<SettingsData>();
    settings.set_location_message(message.into());
    let options: Vec<LocationOption> = places
        .into_iter()
        .map(|place| LocationOption {
            label: place.name.into(),
            latitude: place.latitude as f32,
            longitude: place.longitude as f32,
        })
        .collect();
    settings.set_location_results(ModelRc::from(Rc::new(VecModel::from(options))));
}

fn search_message(result: &anyhow::Result<Vec<GeoPlace>>) -> &'static str {
    match result {
        Ok(places) if places.is_empty() => "Không tìm thấy địa điểm",
        Ok(_) => "Chọn một địa điểm",
        Err(_) => "Không tìm được, kiểm tra kết nối mạng",
    }
}

/// Gọi trên luồng UI. `runtime` chạy yêu cầu mạng của geocoding.
pub fn connect_location(app: &App, input_tx: mpsc::Sender<UserInput>, runtime: Handle) {
    let settings = app.global::<SettingsData>();
    let weak = app.as_weak();
    settings.on_search_location(move |query| {
        let query = query.trim().to_owned();
        let Some(app) = weak.upgrade() else { return };
        if query.is_empty() {
            return;
        }
        show(&app, "Đang tìm…", Vec::new());
        let weak = weak.clone();
        runtime.spawn(async move {
            let result = OpenMeteo::default().search(&query).await;
            let message = search_message(&result);
            let places = result.unwrap_or_default();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = weak.upgrade() {
                    show(&app, message, places);
                }
            });
        });
    });
    let weak = app.as_weak();
    settings.on_pick_location(move |index| {
        let Some(app) = weak.upgrade() else { return };
        let Some(place) = app
            .global::<SettingsData>()
            .get_location_results()
            .row_data(index as usize)
        else {
            return;
        };
        let round = |value: f32| (f64::from(value) * 1e4).round() / 1e4;
        let weather = WeatherSettings {
            latitude: Some(round(place.latitude)),
            longitude: Some(round(place.longitude)),
            name: Some(place.label.to_string()),
        };
        let (reply, _) = oneshot::channel();
        let message = match input_tx.try_send(UserInput::Setup(Update::Weather(weather), reply)) {
            Ok(()) => format!("Đã đặt vị trí nhà: {}", place.label),
            Err(_) => "Không lưu được vị trí".to_owned(),
        };
        show(&app, &message, Vec::new());
        app.global::<SettingsData>().set_location_query("".into());
    });
}
