use hub_core::present::{PhotoState as CorePhotoState, PhotoView, PhotosView};
use slint::{Image, Model, ModelRc, VecModel};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    path::Path,
    rc::Rc,
};

use crate::{Photo, PhotoState};

#[cfg(test)]
#[path = "images_tests.rs"]
mod tests;

thread_local! {
    static IMAGES: RefCell<HashMap<String, CachedImages>> = RefCell::new(HashMap::new());
    static PHOTO_ROWS: Rc<VecModel<Photo>> = Rc::new(VecModel::default());
    static VIEWER_ROWS: Rc<VecModel<Photo>> = Rc::new(VecModel::default());
}

#[derive(Default)]
struct CachedImages {
    thumbnail: CachedImage,
    full: CachedImage,
}

#[derive(Default)]
struct CachedImage {
    path: Option<String>,
    image: Image,
}

pub(super) fn models(view: &PhotosView) -> (ModelRc<Photo>, ModelRc<Photo>) {
    let full_keys = if view.viewing {
        full_image_keys(&view.viewer, view.viewer_index)
    } else {
        HashSet::new()
    };
    let retained: HashSet<&str> = view.photos.iter().map(|photo| photo.key.as_str()).collect();

    let (photos, viewer) = IMAGES.with(|images| {
        let mut images = images.borrow_mut();
        images.retain(|key, _| retained.contains(key.as_str()));

        let photos = view
            .photos
            .iter()
            .map(|photo| model_photo(&mut images, photo, false))
            .collect();
        let viewer = view
            .viewer
            .iter()
            .map(|photo| model_photo(&mut images, photo, full_keys.contains(&photo.key)))
            .collect();
        (photos, viewer)
    });

    PHOTO_ROWS.with(|rows| sync_model(rows, photos));
    VIEWER_ROWS.with(|rows| sync_model(rows, viewer));
    (
        PHOTO_ROWS.with(|rows| ModelRc::from(rows.clone())),
        VIEWER_ROWS.with(|rows| ModelRc::from(rows.clone())),
    )
}

fn sync_model(model: &VecModel<Photo>, rows: Vec<Photo>) {
    while model.row_count() > rows.len() {
        model.remove(rows.len());
    }
    for (row, photo) in rows.into_iter().enumerate() {
        if row < model.row_count() {
            model.set_row_data(row, photo);
        } else {
            model.push(photo);
        }
    }
}

fn model_photo(
    images: &mut HashMap<String, CachedImages>,
    view: &PhotoView,
    load_full: bool,
) -> Photo {
    let cached = images.entry(view.key.clone()).or_default();
    let thumb = cached_image(
        &mut cached.thumbnail,
        &view.thumbnail_path,
        &view.key,
        "thumbnail",
    );
    let full = if load_full {
        cached_image(&mut cached.full, &view.image_path, &view.key, "full")
    } else {
        Image::default()
    };
    Photo {
        key: view.key.as_str().into(),
        thumb,
        full,
        caption: view.caption.as_str().into(),
        source: view.source.as_str().into(),
        time: view.time.as_str().into(),
        pinned: view.pinned,
        state: match view.state {
            CorePhotoState::Loading => PhotoState::Loading,
            CorePhotoState::Ready => PhotoState::Ready,
            CorePhotoState::Failed => PhotoState::Failed,
        },
    }
}

fn cached_image(cache: &mut CachedImage, path: &str, key: &str, kind: &str) -> Image {
    if cache.path.as_deref() != Some(path) {
        cache.path = Some(path.to_owned());
        cache.image = if path.is_empty() {
            Image::default()
        } else {
            match Image::load_from_path(Path::new(path)) {
                Ok(image) => image,
                Err(error) => {
                    eprintln!("không đọc được ảnh {kind} của {key:?} tại {path:?}: {error}");
                    Image::default()
                }
            }
        };
    }
    cache.image.clone()
}

fn full_image_keys(viewer: &[PhotoView], viewer_index: i32) -> HashSet<String> {
    let Some(index) = usize::try_from(viewer_index)
        .ok()
        .filter(|index| *index < viewer.len())
    else {
        return HashSet::new();
    };
    let len = viewer.len();
    [index, (index + len - 1) % len, (index + 1) % len]
        .into_iter()
        .map(|index| viewer[index].key.clone())
        .collect()
}
