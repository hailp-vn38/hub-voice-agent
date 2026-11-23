use crate::{App, PhotosData};
use hub_core::present::PhotosView;
use slint::ComponentHandle;
use std::cell::Cell;

thread_local! {
    static AUTO_ADVANCE: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn apply_photos(app: &App, photos: &PhotosView) {
    let data = app.global::<PhotosData>();
    data.set_title(photos.title.as_str().into());
    data.set_pinned_text(photos.pinned_text.as_str().into());
    data.set_viewing(photos.viewing);
    data.set_viewer_index(photos.viewer_index);
    AUTO_ADVANCE.with(|auto_advance| data.set_auto_advance(auto_advance.get()));

    let (all, viewer) = super::images::models(photos);
    data.set_photos(all);
    data.set_viewer(viewer);
}

pub(super) fn start_auto_advance() {
    AUTO_ADVANCE.with(|auto_advance| auto_advance.set(true));
}

pub(super) fn stop_auto_advance(app: &App) {
    AUTO_ADVANCE.with(|auto_advance| auto_advance.set(false));
    app.global::<PhotosData>().set_auto_advance(false);
}
