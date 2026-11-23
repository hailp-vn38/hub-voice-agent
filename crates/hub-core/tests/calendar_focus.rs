mod support;

use hub_core::{Focus, Screen, UiState, UserInput, present};

#[tokio::test(start_paused = true)]
async fn touching_a_calendar_day_updates_focus_through_run_without_a_focus_serial() {
    let start = "2026-11-23T10:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let mut hub = support::Hub::start(start);
    hub.input_tx
        .send(UserInput::Open(Screen::Calendar))
        .await
        .unwrap();
    loop {
        if hub.ui_rx.borrow().view.screen == Screen::Calendar {
            break;
        }
        hub.ui_rx.changed().await.unwrap();
    }
    hub.input_tx
        .send(UserInput::Focus(Some(Focus::Calendar("2026-11-27".into()))))
        .await
        .unwrap();
    loop {
        if hub.ui_rx.borrow().view.focus == Some(Focus::Calendar("2026-11-27".into())) {
            break;
        }
        hub.ui_rx.changed().await.unwrap();
    }

    let state: &UiState = &hub.ui_rx.borrow();
    let calendar = present(state).calendar;
    assert_eq!(calendar.selected_day, "2026-11-27");
    assert_eq!(calendar.focus, "2026-11-27");
    assert_eq!(state.view.screen, Screen::Calendar);
    assert_eq!(state.events.focus, 0);
    hub.task.abort();
}
