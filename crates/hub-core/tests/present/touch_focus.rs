use hub_core::{Focus, Screen, UserInput};

#[tokio::test(start_paused = true)]
async fn touch_focus_changes_focus_without_changing_screen_or_focus_serial() {
    let hub = crate::support::Hub::start(
        "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
            .parse()
            .unwrap(),
    );
    hub.input_tx
        .send(UserInput::Open(Screen::Calendar))
        .await
        .unwrap();
    tokio::task::yield_now().await;
    let serial = hub.ui_rx.borrow().events.focus;

    hub.input_tx
        .send(UserInput::Focus(Some(Focus::Calendar("2026-11-27".into()))))
        .await
        .unwrap();
    tokio::task::yield_now().await;

    let state = hub.ui_rx.borrow();
    assert_eq!(state.view.screen, Screen::Calendar);
    assert_eq!(state.view.focus, Some(Focus::Calendar("2026-11-27".into())));
    assert_eq!(state.events.focus, serial);
}
