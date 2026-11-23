use hub_core::{
    CalendarEvent, Countdown, Focus, HomeEntity, Photo, Reminder, ReminderRepeat, Repeat, Timer,
    UiState, VoiceSettingsState, WeatherCurrent, WeatherForecast, WeatherView, present,
    present::CenterCard,
};

fn full_state() -> UiState {
    let mut state = UiState::default();
    state.now = "2026-02-17T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let now = state.now.timestamp().as_second();
    state.events.focus = 41;
    state.view.focus = Some(Focus::Timer(4));
    state.weather = WeatherView {
        location: Some("Hà Nội".into()),
        preview: false,
        updated: Some(now - 300),
        forecast: Some(WeatherForecast {
            current: WeatherCurrent {
                temperature: Some(28.4),
                weather_code: Some(61),
                ..WeatherCurrent::default()
            },
            ..WeatherForecast::default()
        }),
        stale: true,
    };
    state.calendar = vec![
        CalendarEvent {
            id: "now".into(),
            summary: "Họp nhóm".into(),
            start: "2026-02-17T11:30:00+07:00".into(),
            end: "2026-02-17T12:30:00+07:00".into(),
            location: None,
            description: None,
        },
        CalendarEvent {
            id: "later".into(),
            summary: "Ăn trưa".into(),
            start: "2026-02-17T13:00:00+07:00".into(),
            end: "2026-02-17T14:00:00+07:00".into(),
            location: None,
            description: None,
        },
        CalendarEvent {
            id: "tomorrow".into(),
            summary: "Ngày mai".into(),
            start: "2026-02-18T09:00:00+07:00".into(),
            end: "2026-02-18T10:00:00+07:00".into(),
            location: None,
            description: None,
        },
    ];
    state.calendar_online = Some(true);
    state.reminders = vec![
        Reminder {
            id: 1,
            label: "Uống thuốc".into(),
            at: "2026-02-17T12:15:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
        Reminder {
            id: 2,
            label: "Gọi mẹ".into(),
            at: "2026-02-17T13:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
    ];
    state.timers = vec![
        Timer {
            id: 4,
            label: "Trứng".into(),
            duration_seconds: 600,
            end_seconds: now + 300,
            paused_seconds: None,
        },
        Timer {
            id: 5,
            label: "Đang tạm dừng".into(),
            duration_seconds: 600,
            end_seconds: now + 60,
            paused_seconds: Some(60),
        },
        Timer {
            id: 6,
            label: "Đã hết".into(),
            duration_seconds: 600,
            end_seconds: now - 1,
            paused_seconds: None,
        },
    ];
    state.countdowns = [
        Countdown {
            id: 10,
            label: "Tết".into(),
            date: "2026-02-17".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2026-02-10T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
        Countdown {
            id: 11,
            label: "Ngày mai".into(),
            date: "2026-02-18".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: "2026-02-10T00:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
        },
    ]
    .into_iter()
    .map(|countdown| countdown.view(state.now.date()).unwrap())
    .collect();
    state.home_configured = true;
    state.home_connected = true;
    state.home = [
        ("light.desk", "Bàn", "off"),
        ("sensor.temp", "Nhiệt độ", "23.5"),
        ("switch.hall", "Hành lang", "on"),
        ("light.kitchen", "Bếp", "off"),
        ("sensor.humidity", "Độ ẩm", "60"),
    ]
    .into_iter()
    .map(|(id, name, value)| HomeEntity {
        id: id.into(),
        name: name.into(),
        state: value.into(),
        unit: None,
        stale: false,
    })
    .collect();
    state.photos = vec![
        Photo {
            id: "p1".into(),
            caption: "Ảnh một".into(),
            source: "example.com".into(),
            image_path: "/p1.png".into(),
            thumbnail_path: "/p1-thumb.png".into(),
            created: now,
            pinned: false,
        },
        Photo {
            id: "p2".into(),
            caption: "Ảnh hai".into(),
            source: "example.com".into(),
            image_path: "/p2.png".into(),
            thumbnail_path: "/p2-thumb.png".into(),
            created: now,
            pinned: false,
        },
    ];
    state.voice_settings = VoiceSettingsState {
        status_label: "Đã kết nối".into(),
        ..VoiceSettingsState::default()
    };
    state
}

#[test]
fn present_prepares_center_cards_from_the_full_snapshot() {
    let view = present(&full_state());
    let center = view.center;

    assert_eq!(center.time, "12:00");
    assert_eq!(center.date, "Thứ Ba, 17/02 · 1/1 âm lịch");
    assert_eq!(center.weather.icon, "rain");
    assert_eq!(center.weather.temp, "28°");
    assert_eq!(center.weather.summary, "Mưa");
    assert_eq!(center.weather.updated, "11:55");
    assert!(center.weather.stale);
    assert_eq!(center.today.len(), 2);
    assert_eq!(center.today[0].key, "now");
    assert_eq!(center.today[0].time, "11:30 – 12:30");
    assert!(center.today[0].now);
    assert!(!center.today[1].now);
    assert_eq!(center.reminders[0].key, "1");
    assert_eq!(center.reminders[0].time, "12:15");
    assert_eq!(center.reminders[0].text, "Uống thuốc");
    assert!(center.timer.visible);
    assert_eq!(center.timer.key, "4");
    assert_eq!(center.timer.label, "Trứng");
    assert_eq!(center.timer.remaining, "5:00");
    assert_eq!(center.timer.progress, 0.5);
    assert!(center.countdown.visible);
    assert_eq!(center.countdown.key, "10");
    assert_eq!(center.countdown.label, "Tết");
    assert_eq!(center.countdown.value, "Hôm nay là Tết");
    assert!(center.countdown.today);
    assert_eq!(center.home.len(), 4);
    assert_eq!(center.home[3].key, "light.kitchen");
    assert!(center.home_safe);
    assert_eq!(center.focus, CenterCard::Timer);
    assert_eq!(center.focus_serial, 41);
}

#[test]
fn present_hides_empty_center_cards_and_menu_lines_follow_feature_data() {
    let empty = present(&UiState::default());
    assert!(!empty.center.timer.visible);
    assert!(!empty.center.countdown.visible);
    assert!(empty.center.weather.temp.is_empty());
    assert!(empty.center.today.is_empty());
    assert!(empty.center.reminders.is_empty());
    assert!(empty.center.home.is_empty());
    assert!(!empty.center.home_safe);
    let mut no_observations = UiState::default();
    no_observations.weather.forecast = Some(WeatherForecast::default());
    let center_weather = present(&no_observations).center.weather;
    assert!(center_weather.temp.is_empty());
    assert!(center_weather.summary.is_empty());

    let mut state = full_state();
    state.home_connected = false;
    assert!(!present(&state).center.home_safe);
    state.home_connected = true;
    let menu = present(&state).menu;
    assert_eq!(menu.weather, "28° · Mưa · cũ");
    assert_eq!(menu.calendar, "2 sự kiện hôm nay");
    assert_eq!(menu.home, "5 thiết bị");
    assert_eq!(menu.timers, "1 hẹn giờ đang chạy");
    assert_eq!(menu.countdowns, "2 mốc sắp tới");
    assert_eq!(menu.reminders, "2 lời nhắc sắp tới");
    assert_eq!(menu.photos, "2 ảnh");
    assert_eq!(menu.settings, "Đã kết nối");

    state.weather.forecast = None;
    state.calendar.clear();
    state.home.clear();
    state.timers.clear();
    state.countdowns.clear();
    state.reminders.clear();
    state.photos.clear();
    state.voice_settings.status_label = "Ngoại tuyến".into();
    let empty_menu = present(&state).menu;
    assert_ne!(empty_menu.weather, menu.weather);
    assert_ne!(empty_menu.calendar, menu.calendar);
    assert_ne!(empty_menu.home, menu.home);
    assert_ne!(empty_menu.timers, menu.timers);
    assert_ne!(empty_menu.countdowns, menu.countdowns);
    assert_ne!(empty_menu.reminders, menu.reminders);
    assert_ne!(empty_menu.photos, menu.photos);
    assert_ne!(empty_menu.settings, menu.settings);
}
