use hub_core::present::{PhotoState, PhotoView};
use std::collections::{HashMap, HashSet};

fn viewer() -> Vec<PhotoView> {
    ["a", "b", "c", "d", "e"]
        .into_iter()
        .map(|key| PhotoView {
            key: key.into(),
            image_path: format!("{key}.png"),
            thumbnail_path: format!("{key}-thumb.png"),
            caption: String::new(),
            source: String::new(),
            time: String::new(),
            pinned: false,
            state: PhotoState::Ready,
        })
        .collect()
}

#[test]
fn full_decode_selection_wraps_at_the_head_middle_and_tail() {
    let photos = viewer();
    assert_eq!(
        super::full_image_keys(&photos, 0),
        HashSet::from(["e".to_string(), "a".to_string(), "b".to_string()]),
    );
    assert_eq!(
        super::full_image_keys(&photos, 2),
        HashSet::from(["b".to_string(), "c".to_string(), "d".to_string()]),
    );
    assert_eq!(
        super::full_image_keys(&photos, 4),
        HashSet::from(["d".to_string(), "e".to_string(), "a".to_string()]),
    );
}

#[test]
fn full_images_outside_the_viewer_window_are_released() {
    let mut images = HashMap::new();
    for key in ["a", "b"] {
        let mut cached = super::CachedImages::default();
        cached.full.path = Some(format!("{key}.png"));
        images.insert(key.to_string(), cached);
    }

    super::release_full_images(&mut images, &HashSet::from(["a".to_string()]));

    assert_eq!(images["a"].full.path.as_deref(), Some("a.png"));
    assert_eq!(images["b"].full.path, None);
}
