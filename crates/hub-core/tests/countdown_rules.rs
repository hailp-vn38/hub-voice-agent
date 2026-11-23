use hub_core::{Countdown, LunarDate, to_solar};
use jiff::civil::date;

#[test]
fn lunar_day_thirty_falls_back_to_day_twenty_nine_in_short_month() {
    assert!(
        to_solar(LunarDate {
            year: 2023,
            month: 4,
            day: 30,
            leap: false
        })
        .is_some()
    );
    let expected = to_solar(LunarDate {
        year: 2024,
        month: 4,
        day: 29,
        leap: false,
    })
    .unwrap();
    assert!(
        to_solar(LunarDate {
            year: 2024,
            month: 4,
            day: 30,
            leap: false
        })
        .is_none()
    );
    let countdown: Countdown = serde_json::from_value(serde_json::json!({"id":1,"label":"Anniversary","date":"2023-04-30","repeat":"yearly_lunar","lunar_leap":false})).unwrap();
    assert_eq!(countdown.next_occurrence(date(2024, 2, 1)), Some(expected));
}

#[test]
fn solar_leap_day_falls_on_february_28_in_common_years() {
    let countdown: Countdown = serde_json::from_value(serde_json::json!({"id":1,"label":"Birthday","date":"2024-02-29","repeat":"yearly","lunar_leap":false})).unwrap();
    assert_eq!(
        countdown.next_occurrence(date(2025, 1, 1)),
        Some(date(2025, 2, 28))
    );
}

#[test]
fn leap_month_anniversary_falls_back_to_regular_month_without_leap() {
    assert!(
        to_solar(LunarDate {
            year: 2023,
            month: 2,
            day: 10,
            leap: true
        })
        .is_some()
    );
    let expected = to_solar(LunarDate {
        year: 2024,
        month: 2,
        day: 10,
        leap: false,
    })
    .unwrap();
    let countdown: Countdown = serde_json::from_value(serde_json::json!({"id":1,"label":"Giỗ","date":"2023-02-10","repeat":"yearly_lunar","lunar_leap":true})).unwrap();
    assert_eq!(countdown.next_occurrence(date(2024, 1, 1)), Some(expected));
}
