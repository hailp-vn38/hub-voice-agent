mod support;
use hub_core::{
    DataKind, FetchedImage, HomeEntity, SettingChange, Store, TimerInput, UiState, UserInput,
    present,
};
use support::Hub;

fn start(store: Store) -> Hub {
    Hub::start_with_store(
        "2026-11-24T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
        store,
    )
}

#[tokio::test]
async fn clearing_timers_removes_them_from_the_ui() {
    let mut hub = start(Store::in_memory().unwrap());
    hub.ui_rx.changed().await.unwrap();
    hub.input_tx
        .send(UserInput::Timer(TimerInput::QuickStart(5)))
        .await
        .unwrap();
    while hub.ui_rx.borrow_and_update().timers.is_empty() {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.input_tx
        .send(UserInput::Clear(DataKind::Timers))
        .await
        .unwrap();
    while !hub.ui_rx.borrow_and_update().timers.is_empty() {
        hub.ui_rx.changed().await.unwrap();
    }
}

#[tokio::test]
async fn clearing_photos_removes_every_photo() {
    let mut store = Store::in_memory().unwrap();
    for n in 0..3 {
        store
            .photos_add(
                format!("p{n}"),
                "example.com".into(),
                FetchedImage {
                    image: vec![1],
                    thumbnail: vec![2],
                },
                n,
            )
            .unwrap();
    }
    let mut hub = start(store);
    while hub.ui_rx.borrow_and_update().photos.len() != 3 {
        hub.ui_rx.changed().await.unwrap();
    }
    hub.input_tx
        .send(UserInput::Clear(DataKind::Photos))
        .await
        .unwrap();
    while !hub.ui_rx.borrow_and_update().photos.is_empty() {
        hub.ui_rx.changed().await.unwrap();
    }
}

fn entity(id: &str) -> HomeEntity {
    HomeEntity {
        id: id.into(),
        name: id.into(),
        state: "on".into(),
        unit: None,
        stale: false,
    }
}

#[test]
fn center_shows_chosen_entities_in_order_or_the_first_four() {
    let mut state = UiState::default();
    state.home = ["light.a", "light.b", "light.c", "light.d", "light.e"]
        .map(entity)
        .to_vec();
    let keys = |state: &UiState| {
        present(state)
            .center
            .home
            .iter()
            .map(|tile| tile.key.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(keys(&state), ["light.a", "light.b", "light.c", "light.d"]);

    for id in ["light.e", "light.b"] {
        state.settings.apply(SettingChange::CenterEntity {
            id: id.into(),
            selected: true,
        });
    }
    assert_eq!(keys(&state), ["light.e", "light.b"]);
}
