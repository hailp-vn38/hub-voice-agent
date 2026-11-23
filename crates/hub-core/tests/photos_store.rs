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
    store
        .photos_add("pinned".into(), "example.com".into(), image(), 0)
        .unwrap();
    let pinned = store.photos_load().unwrap()[0].id.clone();
    store.photos_pin(&pinned, true).unwrap();
    for n in 1..=100 {
        store
            .photos_add(format!("{n}"), "example.com".into(), image(), n)
            .unwrap();
    }
    let photos = store.photos_load().unwrap();
    assert_eq!(photos.len(), 100);
    assert!(photos.iter().any(|p| p.id == pinned));
    assert!(!photos.iter().any(|p| p.caption == "1"));
    assert!(photos.iter().any(|p| p.caption == "100"));
}

#[test]
fn adds_photo_source_column_to_existing_store() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("hub.sqlite3");
    let connection = rusqlite::Connection::open(path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE photos (id TEXT PRIMARY KEY, caption TEXT NOT NULL, created INTEGER NOT NULL, pinned INTEGER NOT NULL, size INTEGER NOT NULL); INSERT INTO photos VALUES ('legacy', 'Old photo', 1, 0, 1);",
        )
        .unwrap();
    drop(connection);

    let store = Store::open(dir.path()).unwrap();

    assert_eq!(store.photos_load().unwrap()[0].source, "");
}
