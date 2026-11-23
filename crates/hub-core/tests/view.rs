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

#[tokio::test(start_paused = true)]
async fn wake_word_exits_ambient() {
    let hub = support::Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.input_tx
        .send(UserInput::Open(Screen::Ambient))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Ambient);
    hub.audio_tx
        .send(hub_core::AudioEvent::WakeWord)
        .await
        .unwrap();
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Center);
}

#[tokio::test(start_paused = true)]
async fn back_returns_to_the_screen_that_opened_the_feature() {
    let hub = support::Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.input_tx
        .send(UserInput::Open(Screen::Menu))
        .await
        .unwrap();
    hub.input_tx
        .send(UserInput::Open(Screen::Timers))
        .await
        .unwrap();
    hub.input_tx.send(UserInput::Back).await.unwrap();
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Menu);
}

#[tokio::test(start_paused = true)]
async fn ambient_begins_five_minutes_after_last_activity() {
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
    tokio::time::advance(Duration::from_secs(60)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Center);
    tokio::time::advance(Duration::from_secs(239)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Center);
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Ambient);
}

#[tokio::test(start_paused = true)]
async fn ambient_timeout_starts_at_actor_start_and_resets_on_activity() {
    let hub = support::Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(200)).await;
    hub.input_tx.send(UserInput::Activity).await.unwrap();
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(299)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Center);
    tokio::time::advance(Duration::from_secs(1)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Ambient);
}

#[tokio::test(start_paused = true)]
async fn ambient_begins_after_five_idle_minutes_from_startup() {
    let hub = support::Hub::start(
        "2026-11-22T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    tokio::task::yield_now().await;
    tokio::time::advance(Duration::from_secs(300)).await;
    tokio::task::yield_now().await;
    assert_eq!(hub.ui_rx.borrow().view.screen, Screen::Ambient);
}
