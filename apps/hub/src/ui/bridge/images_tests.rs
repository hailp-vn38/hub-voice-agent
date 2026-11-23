use hub_core::present::{PhotoState, PhotoView};
use std::collections::HashSet;

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
