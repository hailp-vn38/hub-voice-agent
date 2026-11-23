use hub_core::{Timer, UiState, present};

#[test]
fn present_timer_title_counts_running_timers() {
    let mut state = UiState::default();
    state.now = "2025-11-24T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let now = state.now.timestamp().as_second();
    state.timers = vec![
        Timer {
            id: 1,
            label: "Đang chạy".into(),
            duration_seconds: 300,
            end_seconds: now + 60,
            paused_seconds: None,
        },
        Timer {
            id: 2,
            label: "10 giây cuối".into(),
            duration_seconds: 300,
            end_seconds: now + 10,
            paused_seconds: None,
        },
        Timer {
            id: 3,
            label: "Tạm dừng".into(),
            duration_seconds: 300,
            end_seconds: now + 60,
            paused_seconds: Some(60),
        },
        Timer {
            id: 4,
            label: "Đã hết".into(),
            duration_seconds: 300,
            end_seconds: now - 1,
            paused_seconds: None,
        },
    ];

    assert_eq!(present(&state).timers.title, "Hẹn giờ · 2 đang chạy");
}
