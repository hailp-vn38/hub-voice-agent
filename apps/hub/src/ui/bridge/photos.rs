use crate::{App, PhotosData};
use hub_core::{Focus, UserInput, present::PhotosView};
use slint::{ComponentHandle, Model};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::inputs::Inputs;

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

fn send_focus(inputs: &Rc<RefCell<Inputs>>, focus: Option<Focus>) {
    let mut inputs = inputs.borrow_mut();
    inputs.activity();
    inputs.send(UserInput::Focus(focus));
}

fn adjacent_photo(data: &PhotosData, direction: i32) -> Option<String> {
    let viewer = data.get_viewer();
    let len = viewer.row_count();
    let index = usize::try_from(data.get_viewer_index())
        .ok()
        .filter(|index| *index < len)?;
    let next = (index as i64 + i64::from(direction)).rem_euclid(len as i64) as usize;
    viewer.row_data(next).map(|photo| photo.key.to_string())
}

fn photo_pinned(data: &PhotosData, key: &str) -> Option<bool> {
    let photos = data.get_photos();
    (0..photos.row_count()).find_map(|row| {
        photos
            .row_data(row)
            .filter(|photo| photo.key == key)
            .map(|photo| photo.pinned)
    })
}

pub(super) fn connect_inputs(app: &App, inputs: Rc<RefCell<Inputs>>) {
    let photos = app.global::<PhotosData>();
    let open = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_open(move |key| {
        if let Some(app) = app_weak.upgrade() {
            stop_auto_advance(&app);
        }
        send_focus(&open, Some(Focus::Photo(key.to_string())));
    });

    let next = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_next(move || {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(key) = adjacent_photo(&data, 1) {
                stop_auto_advance(&app);
                send_focus(&next, Some(Focus::Photo(key)));
            }
        }
    });

    let previous = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_previous(move || {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(key) = adjacent_photo(&data, -1) {
                stop_auto_advance(&app);
                send_focus(&previous, Some(Focus::Photo(key)));
            }
        }
    });

    let close = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_close_viewer(move || {
        if let Some(app) = app_weak.upgrade() {
            stop_auto_advance(&app);
        }
        send_focus(&close, None);
    });

    let toggle_pin = inputs.clone();
    let app_weak = app.as_weak();
    photos.on_toggle_pin(move |key| {
        if let Some(app) = app_weak.upgrade() {
            let data = app.global::<PhotosData>();
            if let Some(pinned) = photo_pinned(&data, key.as_str()) {
                stop_auto_advance(&app);
                let mut input = toggle_pin.borrow_mut();
                input.activity();
                input.send(UserInput::PinPhoto {
                    id: key.to_string(),
                    pinned: !pinned,
                });
            }
        }
    });

    let remove = inputs;
    let app_weak = app.as_weak();
    photos.on_remove(move |key| {
        if let Some(app) = app_weak.upgrade() {
            stop_auto_advance(&app);
        }
        let mut input = remove.borrow_mut();
        input.activity();
        input.send(UserInput::RemovePhoto {
            id: key.to_string(),
        });
    });
}
