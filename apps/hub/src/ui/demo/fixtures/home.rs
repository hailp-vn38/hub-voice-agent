use crate::{App, HomeData, HomeKind, HomeTile};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{rc::Rc, time::Duration};

fn tile(key: &str, name: &str, kind: HomeKind, on: bool, value: &str) -> HomeTile {
    HomeTile { key: key.into(), name: name.into(), kind, on, value: value.into(), pending: false, stale: false }
}

pub fn init(app: &App) {
    let h = app.global::<HomeData>();
    h.set_configured(true);
    h.set_connected(true);
    h.set_updated("12:00".into());
    let tiles = vec![
        tile("light.living", "Đèn phòng khách", HomeKind::Light, false, ""),
        tile("light.bed", "Đèn phòng ngủ", HomeKind::Light, true, ""),
        tile("switch.fan", "Quạt", HomeKind::Switch, false, ""),
        tile("sensor.temp", "Nhiệt độ", HomeKind::Sensor, false, "27°C"),
    ];
    h.set_tiles(ModelRc::from(Rc::new(VecModel::from(tiles))));
    let weak = app.as_weak();
    h.on_toggle(move |key| {
        let Some(app) = weak.upgrade() else { return };
        update(&app, &key, |t| t.pending = true);
        let weak = app.as_weak();
        slint::Timer::single_shot(Duration::from_millis(1200), move || {
            if let Some(app) = weak.upgrade() {
                update(&app, &key, |t| {
                    t.pending = false;
                    t.on = !t.on;
                });
            }
        });
    });
}

fn update(app: &App, key: &str, f: impl Fn(&mut HomeTile)) {
    let model = app.global::<HomeData>().get_tiles();
    for i in 0..model.row_count() {
        let mut t = model.row_data(i).unwrap();
        if t.key == key {
            f(&mut t);
            model.set_row_data(i, t);
        }
    }
}

/// Kịch bản "Bật đèn phòng khách": chạm → chờ → bật.
pub fn turn_on_living(app: &App) {
    app.global::<HomeData>().invoke_toggle("light.living".into());
}

/// Kịch bản "Mất kết nối HA": thẻ cũ mờ đi.
pub fn disconnect(app: &App) {
    let h = app.global::<HomeData>();
    h.set_connected(false);
    let model = h.get_tiles();
    for i in 0..model.row_count() {
        let mut t = model.row_data(i).unwrap();
        t.stale = true;
        model.set_row_data(i, t);
    }
}

/// Kịch bản "Chưa cấu hình".
pub fn unconfigure(app: &App) {
    app.global::<HomeData>().set_configured(false);
}
