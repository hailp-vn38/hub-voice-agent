mod support;

use hub_core::{AlertKind, Screen, TimerInput, UiState, UserInput, present};
use std::time::Duration;

async fn wait_for(
    ui: &mut tokio::sync::watch::Receiver<UiState>,
    ready: impl Fn(&UiState) -> bool,
) {
    loop {
        if ready(&ui.borrow()) {
            return;
        }
        ui.changed().await.unwrap();
    }
}

#[tokio::test(start_paused = true)]
async fn touch_timer_controls_change_the_presented_timer() {
    let start: jiff::Zoned = "2026-11-23T14:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let after_pause_seconds = start.timestamp().as_second() + 90;
    let mut hub = support::Hub::start(start);

    hub.input_tx
        .send(UserInput::Timer(TimerInput::QuickStart(5)))
        .await
        .unwrap();
    wait_for(&mut hub.ui_rx, |state| {
        state.view.screen == Screen::Timers && state.timers.len() == 1
    })
    .await;
    let id = hub.ui_rx.borrow().timers[0].id;
    let view = present(&hub.ui_rx.borrow());
    assert_eq!(view.timers.title, "Hẹn giờ");
    assert_eq!(view.timers.items[0].key, id.to_string());
    assert_eq!(view.timers.items[0].label, "5 phút");
    assert_eq!(view.timers.items[0].remaining, "5:00");
    assert_eq!(view.timers.items[0].detail, "5 phút · xong lúc 14:05");
    assert_eq!(view.timers.items[0].progress, 1.0);
    assert_eq!(view.timers.items[0].phase, hub_core::TimerPhase::Running);
    let focus = id.to_string();
    assert_eq!(view.timers.focus.as_deref(), Some(focus.as_str()));
    assert_eq!(view.timers.serial, hub.ui_rx.borrow().events.focus);

    hub.input_tx
        .send(UserInput::Timer(TimerInput::TogglePause(id)))
        .await
        .unwrap();
    wait_for(&mut hub.ui_rx, |state| {
        present(state).timers.items[0].phase == hub_core::TimerPhase::Paused
    })
    .await;
    let paused_remaining = present(&hub.ui_rx.borrow()).timers.items[0]
        .remaining
        .clone();

    tokio::time::advance(Duration::from_secs(90)).await;
    wait_for(&mut hub.ui_rx, |state| {
        state.now.timestamp().as_second() >= after_pause_seconds
    })
    .await;
    assert_eq!(
        present(&hub.ui_rx.borrow()).timers.items[0].remaining,
        paused_remaining
    );

    hub.input_tx
        .send(UserInput::Timer(TimerInput::AddMinute(id)))
        .await
        .unwrap();
    wait_for(&mut hub.ui_rx, |state| {
        present(state).timers.items[0].remaining == "6:00"
    })
    .await;
    hub.input_tx
        .send(UserInput::Timer(TimerInput::Cancel(id)))
        .await
        .unwrap();
    wait_for(&mut hub.ui_rx, |state| state.timers.is_empty()).await;
}

#[tokio::test(start_paused = true)]
async fn old_timer_rows_load_with_remaining_time_as_their_initial_duration() {
    let dir = tempfile::tempdir().unwrap();
    let start: jiff::Zoned = "2026-11-23T14:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let db = rusqlite::Connection::open(dir.path().join("hub.sqlite3")).unwrap();
    db.execute_batch(
        "CREATE TABLE timers (
            id INTEGER PRIMARY KEY,
            label TEXT NOT NULL,
            end_seconds INTEGER NOT NULL,
            paused_seconds INTEGER
        );",
    )
    .unwrap();
    db.execute(
        "INSERT INTO timers (id, label, end_seconds, paused_seconds) VALUES (1, 'tea', ?1, NULL)",
        [start.timestamp().as_second() + 300],
    )
    .unwrap();
    drop(db);

    let mut hub = support::Hub::start_with_store(start, hub_core::Store::open(dir.path()).unwrap());
    wait_for(&mut hub.ui_rx, |state| state.timers.len() == 1).await;
    let timer = &present(&hub.ui_rx.borrow()).timers.items[0];
    assert_eq!(timer.remaining, "5:00");
    assert_eq!(timer.progress, 1.0);
}

#[tokio::test(start_paused = true)]
async fn final_seconds_finish_and_alert_serial_are_presented() {
    let start: jiff::Zoned = "2026-11-23T14:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let start_seconds = start.timestamp().as_second();
    let mut hub = support::Hub::start(start);
    hub.input_tx
        .send(UserInput::Timer(TimerInput::QuickStart(1)))
        .await
        .unwrap();
    wait_for(&mut hub.ui_rx, |state| state.timers.len() == 1).await;

    tokio::time::advance(Duration::from_secs(51)).await;
    wait_for(&mut hub.ui_rx, |state| {
        state.now.timestamp().as_second() >= start_seconds + 51
    })
    .await;
    assert_eq!(
        present(&hub.ui_rx.borrow()).timers.items[0].phase,
        hub_core::TimerPhase::FinalSeconds
    );
    assert_eq!(
        present(&hub.ui_rx.borrow()).timers.items[0].remaining,
        "0:09"
    );

    tokio::time::advance(Duration::from_secs(9)).await;
    wait_for(&mut hub.ui_rx, |state| state.alert.is_some()).await;
    let view = present(&hub.ui_rx.borrow());
    assert_eq!(view.timers.items[0].phase, hub_core::TimerPhase::Finished);
    let alert = view.alert.as_ref().unwrap();
    assert_eq!(alert.kind, AlertKind::Timer);
    assert_eq!(alert.title, "Hẹn giờ đã xong");
    assert_eq!(alert.time, "14:01");
    assert_eq!(alert.body, "1 phút");
    assert_eq!(alert.serial, hub.ui_rx.borrow().events.alert);
    assert!(alert.serial > 0);
}
