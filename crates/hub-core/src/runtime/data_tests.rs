use super::super::effects::Effect;
use crate::{
    Countdown, Reminder, ReminderRepeat, Repeat, Store, UiState,
    features::{countdowns, reminders},
};

/// Chạy các effect UI và ghi store như runtime thật.
fn run(effects: Vec<Effect>, store: &mut Store, ui: &mut UiState) {
    for effect in effects {
        match effect {
            Effect::Ui(patch) => {
                patch(ui);
            }
            Effect::Store(write) => write(store).unwrap(),
            _ => {}
        }
    }
}

#[test]
fn clearing_countdowns_empties_state_and_storage() {
    let mut store = Store::in_memory().unwrap();
    store
        .countdowns_save(&[Countdown {
            id: 1,
            label: "Tết".into(),
            date: "2027-02-06".into(),
            repeat: Repeat::None,
            lunar_leap: false,
            created_at: String::new(),
        }])
        .unwrap();
    let mut feature = countdowns::Feature::load(&store).unwrap();
    let mut ui = UiState::default();
    let today = "2026-11-24".parse().unwrap();
    run(feature.snapshot(today), &mut store, &mut ui);
    assert_eq!(ui.countdowns.len(), 1);

    run(feature.clear(today), &mut store, &mut ui);

    assert!(ui.countdowns.is_empty());
    assert!(store.countdowns_load().unwrap().is_empty());
}

#[test]
fn clearing_reminders_drops_items_and_history() {
    let mut store = Store::in_memory().unwrap();
    let reminder = Reminder {
        id: 1,
        label: "Uống thuốc".into(),
        at: "2026-11-25T08:00".into(),
        repeat: ReminderRepeat::None,
        weekdays: Vec::new(),
        scheduled_at: None,
    };
    store.reminders_save(&[reminder], &[]).unwrap();
    let mut feature = reminders::Feature::load(&store).unwrap();
    let mut ui = UiState::default();
    run(feature.snapshot(), &mut store, &mut ui);
    assert_eq!(ui.reminders.len(), 1);

    run(feature.clear(), &mut store, &mut ui);

    assert!(ui.reminders.is_empty());
    assert!(store.reminders_load().unwrap().is_empty());
}

#[test]
fn clearing_the_weather_cache_removes_saved_forecasts() {
    let store = Store::in_memory().unwrap();
    store
        .weather_save("home", 1, &serde_json::json!({"a": 1}))
        .unwrap();
    store.weather_clear().unwrap();
    assert!(store.weather_get("home").unwrap().is_none());
}
