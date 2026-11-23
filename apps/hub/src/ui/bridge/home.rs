use crate::{App, HomeData, HomeKind, HomeTile};
use hub_core::present::{HomeKind as CoreHomeKind, HomeView};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::rc::Rc;

thread_local! {
    static HOME_TILES: Rc<VecModel<HomeTile>> = Rc::new(VecModel::default());
}

pub(super) fn apply_home(app: &App, home: &HomeView) {
    let data = app.global::<HomeData>();
    data.set_configured(home.configured);
    data.set_connected(home.connected);
    data.set_updated(home.updated.as_str().into());

    HOME_TILES.with(|tiles| {
        while tiles.row_count() > home.tiles.len() {
            tiles.remove(home.tiles.len());
        }
        for (row, tile) in home.tiles.iter().enumerate() {
            let tile = HomeTile {
                key: tile.key.as_str().into(),
                name: tile.name.as_str().into(),
                kind: match tile.kind {
                    CoreHomeKind::Light => HomeKind::Light,
                    CoreHomeKind::Switch => HomeKind::Switch,
                    CoreHomeKind::Sensor => HomeKind::Sensor,
                },
                on: tile.on,
                value: tile.value.as_str().into(),
                pending: tile.pending,
                stale: tile.stale,
            };
            if row < tiles.row_count() {
                tiles.set_row_data(row, tile);
            } else {
                tiles.push(tile);
            }
        }
        data.set_tiles(ModelRc::from(tiles.clone()));
    });
}
