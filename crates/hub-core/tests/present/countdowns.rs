use hub_core::{Countdown, Repeat, UiState, present, present::CountdownUnit};

#[test]
fn present_prepares_countdowns_from_the_snapshot_time() {
    let mut state = UiState::default();
    state.now = "2025-11-24T21:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.events.confetti = 7;
    let today = state.now.date();
    state.countdowns = [
        Countdown {
            id: 1,
            label: "Hôm nay".into(),
            date: "2025-11-24".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2025-11-20T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
        Countdown {
            id: 2,
            label: "Sắp tới".into(),
            date: "2025-11-25".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2025-11-20T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
        Countdown {
            id: 3,
            label: "Đã qua".into(),
            date: "2025-11-20".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2025-11-19T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
        Countdown {
            id: 4,
            label: "Tết".into(),
            date: "2026-1-1".into(),
            repeat: Repeat::YearlyLunar,
            lunar_leap: false,
            created_at: "2025-11-20T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
        Countdown {
            id: 5,
            label: "Chuyến đi".into(),
            date: "2025-11-28".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2025-11-20T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
    ]
    .into_iter()
    .map(|countdown| countdown.view(today).unwrap())
    .collect();
    state.countdowns.swap(0, 1);

    let view = present(&state);
    let countdowns = view.countdowns;
    assert_eq!(countdowns.title, "Ngày đặc biệt · 5 mốc");
    assert!(countdowns.has_main);
    assert_eq!(countdowns.main.key, "1");
    assert_eq!(countdowns.main.unit, CountdownUnit::Today);
    assert_eq!(countdowns.main.value, "Hôm nay là Hôm nay");
    assert_eq!(countdowns.main.target, "Thứ Hai 24/11");
    assert_eq!(countdowns.confetti_serial, 7);

    let live = countdowns
        .others
        .iter()
        .find(|countdown| countdown.key == "2")
        .unwrap();
    assert_eq!(live.unit, CountdownUnit::Live);
    assert_eq!(live.value, "03:00:00");
    assert_eq!(live.repeat_text, "");

    let passed = countdowns
        .others
        .iter()
        .find(|countdown| countdown.key == "3")
        .unwrap();
    assert_eq!(passed.unit, CountdownUnit::Passed);
    assert_eq!(passed.value, "Đã qua");

    let lunar = countdowns
        .others
        .iter()
        .find(|countdown| countdown.key == "4")
        .unwrap();
    assert_eq!(lunar.lunar, "1/1 âm lịch");
    assert_eq!(lunar.repeat_text, "Hằng năm âm lịch");

    let trip = countdowns
        .others
        .iter()
        .find(|countdown| countdown.key == "5")
        .unwrap();
    assert_eq!(trip.unit, CountdownUnit::Days);
    assert_eq!(trip.progress, 0.609375);
    assert_eq!(trip.progress_text, "61% chặng đường");
}
