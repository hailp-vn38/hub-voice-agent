use hub_core::{FetchedImage, Store};

fn image() -> FetchedImage {
    FetchedImage {
        image: vec![1],
        thumbnail: vec![2],
    }
}

#[test]
fn evicts_oldest_unpinned_photo() {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store.photos_add("pinned".into(), image(), 0).unwrap();
    let pinned = store.photos_load().unwrap()[0].id.clone();
    store.photos_pin(&pinned, true).unwrap();
    for n in 1..=100 {
        store.photos_add(format!("{n}"), image(), n).unwrap();
    }
    let photos = store.photos_load().unwrap();
    assert_eq!(photos.len(), 100);
    assert!(photos.iter().any(|p| p.id == pinned));
    assert!(!photos.iter().any(|p| p.caption == "1"));
    assert!(photos.iter().any(|p| p.caption == "100"));
}
