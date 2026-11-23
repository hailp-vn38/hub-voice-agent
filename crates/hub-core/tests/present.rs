mod support;

use futures_util::{SinkExt, StreamExt};
use hub_core::{
    Countdown, Focus, Reminder, ReminderHistory, ReminderRepeat, Repeat, Screen, UiState,
    UserInput, VoiceState, WeatherCurrent, WeatherDay, WeatherForecast, present,
    present::CountdownUnit,
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::{mpsc, watch},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

#[test]
fn present_maps_nav_status_and_ambient_from_the_snapshot_time() {
    let mut state = UiState::default();
    state.now = "2026-11-23T05:45:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.view.screen = Screen::Calendar;
    state.voice.state = VoiceState::Offline;
    state.weather.forecast = Some(WeatherForecast {
        current: WeatherCurrent {
            temperature: Some(28.9),
            weather_code: Some(61),
            ..WeatherCurrent::default()
        },
        days: vec![WeatherDay {
            date: "2026-11-23".into(),
            sunrise: Some("2026-11-23T06:10".into()),
            sunset: Some("2026-11-23T17:20".into()),
            ..WeatherDay::default()
        }],
        ..WeatherForecast::default()
    });

    let view = present(&state);

    assert_eq!(view.nav, Screen::Calendar);
    assert_eq!(view.status.time, "05:45");
    assert!(view.status.offline);
    assert_eq!(view.ambient.time, "05:45");
    assert_eq!(view.ambient.temp, "29°");
    assert_eq!(view.ambient.icon, "rain");
    assert!(view.ambient.night);
    assert!(!view.weather.is_day);
}

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
    assert_eq!(countdowns.title, "Đếm ngược · 5 mốc");
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

#[test]
fn present_groups_reminders_and_formats_until_and_recent_history() {
    let mut state = UiState::default();
    state.now = "2025-11-24T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    state.reminders = vec![
        Reminder {
            id: 1,
            label: "Uống thuốc".into(),
            at: "2025-11-24T12:25:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::Daily,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
        Reminder {
            id: 2,
            label: "Khám răng".into(),
            at: "2025-11-25T09:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
        Reminder {
            id: 3,
            label: "Gọi mẹ".into(),
            at: "2025-11-28T18:00:00+07:00[Asia/Ho_Chi_Minh]".into(),
            repeat: ReminderRepeat::None,
            weekdays: Vec::new(),
            scheduled_at: None,
        },
    ];
    let past = state.now.checked_sub(jiff::Span::new().minutes(5)).unwrap();
    state.reminder_history.push(ReminderHistory {
        reminder: state.reminders[1].clone(),
        missed: true,
        at_seconds: past.timestamp().as_second(),
    });

    let reminders = present(&state).reminders;
    assert_eq!(reminders.title, "3 sắp tới");
    assert!(reminders.has_next);
    assert_eq!(reminders.next.key, "1");
    assert_eq!(reminders.next.time, "12:25");
    assert_eq!(reminders.next.until, "còn 25 phút");
    assert_eq!(reminders.groups[0].label, "Hôm nay");
    assert_eq!(reminders.groups[0].items[0].repeat_text, "Hằng ngày");
    assert!(reminders.groups[0].items[0].repeats);
    assert_eq!(reminders.groups[1].label, "Ngày mai");
    assert_eq!(reminders.groups[1].items[0].repeat_text, "");
    assert!(!reminders.groups[1].items[0].repeats);
    assert_eq!(reminders.groups[2].label, "Thứ Sáu 28/11");
    assert_eq!(reminders.recent[0].time, "11:55");
    assert!(reminders.recent[0].missed);

    state.now = "2025-11-24T12:05:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    assert_eq!(present(&state).reminders.next.until, "còn 20 phút");
}

#[tokio::test(start_paused = true)]
async fn touch_focus_changes_focus_without_changing_screen_or_focus_serial() {
    let hub = support::Hub::start(
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

#[tokio::test(start_paused = true)]
async fn repeated_timer_start_advances_focus_serial_for_each_focus() {
    let ws = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ws_addr = ws.local_addr().unwrap();
    let ota = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ota_addr = ota.local_addr().unwrap();
    let ota_task = tokio::spawn(async move {
        let (mut conn, _) = ota.accept().await.unwrap();
        let mut buf = [0; 4096];
        assert!(conn.read(&mut buf).await.unwrap() > 0);
        let body = json!({
            "websocket": {"url": format!("ws://{ws_addr}"), "token": "token"}
        })
        .to_string();
        conn.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    });

    let dir = tempfile::tempdir().unwrap();
    let mut config = hub_core::ConfigStore::open(dir.path()).unwrap();
    config
        .update(|settings| {
            settings.voice.ota_url = Some(format!("http://{ota_addr}/voice/ota/"));
        })
        .unwrap();
    let (voice_config_tx, voice_config_rx) = watch::channel(None);
    let (voice_tx, voice_rx) = voice_client::spawn(voice_config_rx);
    let (audio_tx, _audio_commands) = mpsc::channel(16);
    let (_audio_events, audio_rx) = mpsc::channel(16);
    let (_input_sender, input_rx) = mpsc::channel(16);
    let (ui_tx, ui_rx) = watch::channel(UiState::default());
    let start = "2026-11-23T12:00:00+07:00[Asia/Ho_Chi_Minh]"
        .parse()
        .unwrap();
    let task = tokio::spawn(hub_core::run(
        config,
        hub_core::Store::in_memory().unwrap(),
        hub_core::Ports {
            clock: Arc::new(support::TestClock::at(start)),
            calendar: Arc::new(hub_core::GoogleCalendar::default()),
            weather: Arc::new(hub_core::OpenMeteo::default()),
            home: Arc::new(hub_core::HomeAssistant::default()),
            images: Arc::new(hub_core::HttpImageFetcher),
            system: Arc::new(support::Synced),
        },
        hub_core::Io {
            voice_config_tx,
            voice_tx,
            voice_rx,
            audio_tx,
            audio_rx,
            input_rx,
            ui_tx,
        },
    ));

    let (stream, _) = ws.accept().await.unwrap();
    let mut socket = accept_async(stream).await.unwrap();
    socket.next().await.unwrap().unwrap();
    socket
        .send(Message::Text(
            json!({
                "type": "hello",
                "transport": "websocket",
                "session_id": "test",
                "audio_params": {
                    "format": "opus",
                    "sample_rate": 24000,
                    "channels": 1,
                    "frame_duration": 60
                }
            })
            .to_string()
            .into(),
        ))
        .await
        .unwrap();

    for id in 1..=2 {
        socket
            .send(Message::Text(
                json!({
                    "type": "mcp",
                    "payload": {
                        "jsonrpc": "2.0",
                        "id": id,
                        "method": "tools/call",
                        "params": {
                            "name": "hub_timer_start",
                            "arguments": {"minutes": 1, "label": "luộc trứng"}
                        }
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .unwrap();
        let reply: Value = loop {
            let message = tokio::time::timeout(Duration::from_secs(5), socket.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            if let Message::Text(text) = message {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["payload"]["id"] == id {
                    break value;
                }
            }
        };
        assert!(reply["payload"]["result"].is_object(), "{reply}");
        // On the second call, show replays the old focus before start focuses the new timer.
        let expected_serial = if id == 1 { 1 } else { 3 };
        assert_eq!(ui_rx.borrow().events.focus, expected_serial);
    }

    assert_eq!(ui_rx.borrow().view.screen, Screen::Timers);
    assert_eq!(ui_rx.borrow().timers.len(), 2);
    assert_eq!(ui_rx.borrow().view.focus, Some(Focus::Timer(2)));
    drop(ui_rx);
    task.await.unwrap().unwrap();
    ota_task.await.unwrap();
}
