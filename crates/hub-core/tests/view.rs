mod support;

use hub_core::{Screen, UserInput};
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn navigation_resets_return_deadline() {
    let hub = support::Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.input_tx
        .send(UserInput::Open(Screen::Timers))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Timers);
    tokio::time::advance(Duration::from_secs(40)).await;
    hub.input_tx.send(UserInput::Activity).await.unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(40)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Timers);
    tokio::time::advance(Duration::from_secs(20)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Center);
}
