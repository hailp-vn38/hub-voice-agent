#[allow(dead_code)]
mod support;
use hub_core::{
    CalendarDelta, CalendarEvent, CalendarFuture, CalendarSettings, CalendarSource, Io,
    NewCalendarEvent, Ports, Store, UiState, run,
};
use std::sync::{Arc, Mutex};
use support::TestClock;
use tokio::sync::{mpsc, watch};

#[derive(Default)]
struct Fake {
    calls: Mutex<Vec<Option<String>>>,
}
impl CalendarSource for Fake {
    fn list(&self, _: CalendarSettings, token: Option<String>) -> CalendarFuture<CalendarDelta> {
        let count = {
            let mut calls = self.calls.lock().unwrap();
            calls.push(token.clone());
            calls.len()
        };
        Box::pin(async move {
            if count >= 4 {
                anyhow::bail!("offline");
            }
            if token.as_deref() == Some("first") {
                return Ok(CalendarDelta {
                    expired: true,
                    ..Default::default()
                });
            }
            if token.as_deref() == Some("second") {
                anyhow::bail!("offline");
            }
            Ok(CalendarDelta {
                events: vec![CalendarEvent {
                    id: "one".into(),
                    summary: "Meeting".into(),
                    start: "2026-11-23T10:00:00+07:00".into(),
                    end: "2026-11-23T11:00:00+07:00".into(),
                }],
                sync_token: Some(if token.is_none() { "first" } else { "second" }.into()),
                ..Default::default()
            })
        })
    }
    fn create(&self, _: CalendarSettings, _: NewCalendarEvent) -> CalendarFuture<CalendarEvent> {
        Box::pin(async { anyhow::bail!("unused") })
    }
}
#[tokio::test(start_paused = true)]
async fn actor_refreshes_and_keeps_cached_events_when_offline() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|c| c.calendar.refresh_token = "refresh".into())
        .unwrap();
    let fake = Arc::new(Fake::default());
    let (voice_config_tx, _) = watch::channel(None);
    let (voice_tx, _voice_rx) = mpsc::channel(16);
    let (_voice_events, voice_rx) = mpsc::channel(16);
    let (audio_tx, _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let task = tokio::spawn(run(
        config,
        Store::open(dir.path()).unwrap(),
        Ports {
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            clock: Arc::new(TestClock::at(
                "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
                    .parse()
                    .unwrap(),
            )),
            calendar: fake.clone(),
            system: Arc::new(support::Synced),
            weather: Arc::new(hub_core::OpenMeteo::default()),
        },
        Io {
            voice_config_tx,
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));
    while ui_rx.borrow().calendar.is_empty() {
        ui_rx.changed().await.unwrap();
    }
    assert_eq!(ui_rx.borrow().calendar[0].summary, "Meeting");
    tokio::time::advance(std::time::Duration::from_secs(301)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert_eq!(
        fake.calls.lock().unwrap().as_slice(),
        &[None, Some("first".into()), None]
    );
    assert_eq!(ui_rx.borrow().calendar[0].id, "one");
    tokio::time::advance(std::time::Duration::from_secs(301)).await;
    for _ in 0..20 {
        tokio::task::yield_now().await;
    }
    assert_eq!(fake.calls.lock().unwrap().len(), 4);
    assert_eq!(ui_rx.borrow().calendar[0].id, "one");
    task.abort();
}

#[derive(Default)]
struct FutureEvent;
impl CalendarSource for FutureEvent {
    fn list(&self, _: CalendarSettings, token: Option<String>) -> CalendarFuture<CalendarDelta> {
        Box::pin(async move {
            Ok(CalendarDelta {
                events: if token.is_none() {
                    vec![CalendarEvent {
                        id: "future".into(),
                        summary: "Future".into(),
                        start: "2027-01-23T10:00:00Z".into(),
                        end: "2027-01-23T11:00:00Z".into(),
                    }]
                } else {
                    vec![]
                },
                sync_token: Some("token".into()),
                ..Default::default()
            })
        })
    }
    fn create(&self, _: CalendarSettings, _: NewCalendarEvent) -> CalendarFuture<CalendarEvent> {
        Box::pin(async { anyhow::bail!("unused") })
    }
}
#[tokio::test(start_paused = true)]
async fn future_event_enters_rolling_cache_without_remote_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|c| c.calendar.refresh_token = "refresh".into())
        .unwrap();
    let (voice_config_tx, _) = watch::channel(None);
    let (voice_tx, _voice_rx) = mpsc::channel(16);
    let (_voice_events, voice_rx) = mpsc::channel(16);
    let (audio_tx, _audio_rx) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_tx, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let task = tokio::spawn(run(
        config,
        Store::open(dir.path()).unwrap(),
        Ports {
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            clock: Arc::new(TestClock::at("2026-11-23T12:00:00Z[UTC]".parse().unwrap())),
            calendar: Arc::new(FutureEvent),
            system: Arc::new(support::Synced),
            weather: Arc::new(hub_core::OpenMeteo::default()),
        },
        Io {
            voice_config_tx,
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));
    for _ in 0..30 {
        tokio::task::yield_now().await;
    }
    assert!(ui_rx.borrow().calendar.is_empty());
    tokio::time::advance(std::time::Duration::from_secs(2 * 86400)).await;
    for _ in 0..50 {
        tokio::task::yield_now().await;
    }
    assert_eq!(ui_rx.borrow().calendar[0].id, "future");
    task.abort();
}
