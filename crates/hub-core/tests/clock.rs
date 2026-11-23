mod support;

use std::time::Duration;

use support::Hub;

#[tokio::test(start_paused = true)]
async fn clock_in_ui_state_advances_each_second() {
    let start = "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let mut hub = Hub::start(start);

    hub.ui_rx.changed().await.unwrap();
    assert_eq!(hub.ui_rx.borrow_and_update().clock, "12:00:00");

    tokio::time::sleep(Duration::from_secs(1)).await;
    hub.ui_rx.changed().await.unwrap();
    assert_eq!(hub.ui_rx.borrow_and_update().clock, "12:00:01");

    drop(hub.ui_rx);
    hub.task.await.unwrap().unwrap();
}
