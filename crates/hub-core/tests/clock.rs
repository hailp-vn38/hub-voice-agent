mod support;

use std::{sync::Arc, time::Duration};

use hub_core::{Io, Ports, UiState, run};
use support::TestClock;
use tokio::sync::watch;

#[tokio::test(start_paused = true)]
async fn clock_in_ui_state_advances_each_second() {
    let start: jiff::Zoned = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let ports = Ports {
        clock: Arc::new(TestClock::at(start)),
    };
    let (ui_tx, mut ui_rx) = watch::channel(UiState::default());
    let core = tokio::spawn(run(ports, Io { ui_tx }));

    ui_rx.changed().await.unwrap();
    assert_eq!(ui_rx.borrow_and_update().clock, "12:00:00");

    tokio::time::sleep(Duration::from_secs(1)).await;
    ui_rx.changed().await.unwrap();
    assert_eq!(ui_rx.borrow_and_update().clock, "12:00:01");

    drop(ui_rx);
    core.await.unwrap().unwrap();
}
