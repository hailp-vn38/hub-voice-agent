#[allow(dead_code)]
mod support;

use hub_core::{
    HomeEntity, HomeEvent, HomeSettings, HomeSource, Io, Ports, Screen, Store, UiState, UserInput,
    present, run,
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch};

struct FakeHome {
    events: Mutex<Option<mpsc::Sender<HomeEvent>>>,
    calls: mpsc::UnboundedSender<(String, String)>,
}

impl HomeSource for FakeHome {
    fn subscribe(&self, _: watch::Receiver<HomeSettings>) -> mpsc::Receiver<HomeEvent> {
        let (tx, rx) = mpsc::channel(16);
        *self.events.lock().unwrap() = Some(tx);
        rx
    }

    fn call_service(
        &self,
        entity_id: String,
        service: String,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send + '_>> {
        Box::pin(async move {
            self.calls.send((entity_id, service))?;
            Ok(())
        })
    }
}

async fn send_home(fake: &FakeHome, event: HomeEvent) {
    let sender = loop {
        if let Some(sender) = fake.events.lock().unwrap().clone() {
            break sender;
        }
        tokio::task::yield_now().await;
    };
    sender.send(event).await.unwrap();
}

async fn wait_for(ui: &mut watch::Receiver<UiState>, mut ready: impl FnMut(&UiState) -> bool) {
    loop {
        if ready(&ui.borrow()) {
            return;
        }
        ui.changed().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn touch_toggle_stays_pending_until_home_sends_a_new_state() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|config| {
            config.home = HomeSettings {
                url: "http://home.local".into(),
                token: "secret".into(),
            };
        })
        .unwrap();
    let (calls_tx, mut calls_rx) = mpsc::unbounded_channel();
    let fake = Arc::new(FakeHome {
        events: Mutex::new(None),
        calls: calls_tx,
    });
    let (voice_tx, _voice_commands) = mpsc::channel(16);
    let (_voice_events, voice_rx) = mpsc::channel(16);
    let (audio_tx, _audio_commands) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let task = tokio::spawn(run(
        config,
        Store::in_memory().unwrap(),
        Ports {
            device: std::sync::Arc::new(hub_core::NoDevice),
            clock: Arc::new(support::TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: Arc::new(hub_core::GoogleCalendar::default()),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: fake.clone(),
            images: Arc::new(hub_core::HttpImageFetcher),
            system: Arc::new(support::Synced),
        },
        Io {
            voice_config_tx: watch::channel(None).0,
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));

    send_home(&fake, HomeEvent::Connected).await;
    send_home(
        &fake,
        HomeEvent::State(HomeEntity {
            id: "light.desk".into(),
            name: "Desk".into(),
            state: "off".into(),
            unit: None,
            stale: false,
        }),
    )
    .await;
    wait_for(&mut ui_rx, |state| state.home.len() == 1).await;
    assert!(ui_rx.borrow().home_connected);
    assert!(ui_rx.borrow().home_configured);
    assert_eq!(present(&ui_rx.borrow()).home.updated, "12:00");

    input_tx
        .send(UserInput::ToggleHome("light.desk".into()))
        .await
        .unwrap();
    assert_eq!(
        calls_rx.recv().await.unwrap(),
        ("light.desk".into(), "turn_on".into())
    );
    wait_for(&mut ui_rx, |state| present(state).home.tiles[0].pending).await;

    send_home(
        &fake,
        HomeEvent::State(HomeEntity {
            id: "light.desk".into(),
            name: "Desk".into(),
            state: "on".into(),
            unit: None,
            stale: false,
        }),
    )
    .await;
    wait_for(&mut ui_rx, |state| {
        let tile = &present(state).home.tiles[0];
        tile.on && !tile.pending
    })
    .await;

    drop(ui_rx);
    task.abort();
}

#[test]
fn present_formats_home_sensor_values_and_photo_view_state() {
    let mut state = UiState::default();
    state.now = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.view.screen = Screen::Photos;
    state.view.focus = Some(hub_core::Focus::Photo("p2".into()));
    state.home.push(HomeEntity {
        id: "sensor.temp".into(),
        name: "Nhiệt độ".into(),
        state: "23.5".into(),
        unit: Some("°C".into()),
        stale: false,
    });
    state.photos.push(hub_core::Photo {
        id: "p1".into(),
        caption: "Hồ".into(),
        source: "example.com".into(),
        image_path: "/photos/p1.png".into(),
        thumbnail_path: "/photos/p1-thumb.png".into(),
        created: state.now.timestamp().as_second(),
        pinned: true,
    });
    state.photos.push(hub_core::Photo {
        id: "p2".into(),
        caption: "Núi".into(),
        source: "images.example".into(),
        image_path: "/photos/p2.png".into(),
        thumbnail_path: "/photos/p2-thumb.png".into(),
        created: state.now.timestamp().as_second(),
        pinned: false,
    });

    let view = present(&state);

    assert_eq!(view.home.tiles[0].value, "23,5 °C");
    assert!(view.photos.viewing);
    assert_eq!(view.photos.viewer_index, 1);
    assert_eq!(view.photos.photos[0].source, "example.com");
    assert_eq!(view.photos.photos[0].time, "12:00");
    assert_eq!(view.photos.photos[0].state, hub_core::PhotoState::Ready);
    assert_eq!(view.photos.title, "Ảnh · 2 ảnh");
    assert_eq!(view.photos.pinned_text, "1 đã giữ");
}

#[test]
fn present_marks_photo_fetches_loading_and_failed() {
    let mut state = UiState::default();
    state.photo_fetches = vec![
        hub_core::PhotoFetch {
            key: "loading".into(),
            caption: "Đang tải".into(),
            source: "example.com".into(),
            created: 0,
            state: hub_core::PhotoFetchState::Loading,
        },
        hub_core::PhotoFetch {
            key: "failed".into(),
            caption: "Lỗi tải".into(),
            source: "example.com".into(),
            created: 0,
            state: hub_core::PhotoFetchState::Failed,
        },
    ];

    let view = present(&state);

    assert_eq!(view.photos.photos[0].state, hub_core::PhotoState::Loading);
    assert_eq!(view.photos.photos[1].state, hub_core::PhotoState::Failed);
}
