use crate::{Focus, Photo, PhotoFetch, PhotoFetchState, Screen, UiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhotosView {
    pub photos: Vec<PhotoView>,
    pub viewing: bool,
    pub viewer: Vec<PhotoView>,
    pub viewer_index: i32,
    pub title: String,
    pub pinned_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhotoView {
    pub key: String,
    pub image_path: String,
    pub thumbnail_path: String,
    pub caption: String,
    pub source: String,
    pub time: String,
    pub pinned: bool,
    pub state: PhotoState,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoState {
    Loading,
    Ready,
    Failed,
}

pub(super) fn from_state(state: &UiState) -> PhotosView {
    let mut photos: Vec<_> = state
        .photo_fetches
        .iter()
        .map(|fetch| (fetch.created, from_fetch(fetch, state)))
        .collect();
    photos.extend(
        state
            .photos
            .iter()
            .map(|photo| (photo.created, from_photo(photo, state))),
    );
    photos.sort_by(|left, right| right.0.cmp(&left.0));
    let photos: Vec<_> = photos.into_iter().map(|(_, photo)| photo).collect();

    let viewer: Vec<_> = state
        .photos
        .iter()
        .map(|photo| from_photo(photo, state))
        .collect();
    let focus_id = match state.view.focus.as_ref() {
        Some(Focus::Photo(id)) => Some(id.as_str()),
        _ => None,
    };
    let viewing = state.view.screen == Screen::Photos && focus_id.is_some();
    let viewer_index = focus_id
        .and_then(|id| viewer.iter().position(|photo| photo.key == id))
        .unwrap_or_default()
        .try_into()
        .unwrap_or(i32::MAX);

    PhotosView {
        title: format!("Ảnh · {} ảnh", photos.len()),
        pinned_text: format!(
            "{} đã giữ",
            state.photos.iter().filter(|photo| photo.pinned).count()
        ),
        photos,
        viewing,
        viewer,
        viewer_index,
    }
}

fn from_photo(photo: &Photo, state: &UiState) -> PhotoView {
    PhotoView {
        key: photo.id.clone(),
        image_path: photo.image_path.clone(),
        thumbnail_path: photo.thumbnail_path.clone(),
        caption: photo.caption.clone(),
        source: photo.source.clone(),
        time: super::timers::time_at(&state.now, photo.created),
        pinned: photo.pinned,
        state: PhotoState::Ready,
    }
}

fn from_fetch(fetch: &PhotoFetch, state: &UiState) -> PhotoView {
    PhotoView {
        key: fetch.key.clone(),
        image_path: String::new(),
        thumbnail_path: String::new(),
        caption: fetch.caption.clone(),
        source: fetch.source.clone(),
        time: super::timers::time_at(&state.now, fetch.created),
        pinned: false,
        state: match fetch.state {
            PhotoFetchState::Loading => PhotoState::Loading,
            PhotoFetchState::Failed => PhotoState::Failed,
        },
    }
}
