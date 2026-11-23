use hub_core::{LunarDate, to_lunar, to_solar};
use jiff::civil::date;

#[test]
fn tet_2020_through_2030_and_leap_month_round_trip() {
    let dates = [
        (2020, 1, 25),
        (2021, 2, 12),
        (2022, 2, 1),
        (2023, 1, 22),
        (2024, 2, 10),
        (2025, 1, 29),
        (2026, 2, 17),
        (2027, 2, 6),
        (2028, 1, 26),
        (2029, 2, 13),
        (2030, 2, 2),
    ];
    for (year, month, day) in dates {
        let solar = date(year, month, day);
        let lunar = LunarDate {
            year,
            month: 1,
            day: 1,
            leap: false,
        };
        assert_eq!(to_lunar(solar), lunar);
        assert_eq!(to_solar(lunar), Some(solar));
    }
    let leap = LunarDate {
        year: 2023,
        month: 2,
        day: 1,
        leap: true,
    };
    assert_eq!(to_solar(leap), Some(date(2023, 3, 22)));
    assert_eq!(to_lunar(date(2023, 3, 22)), leap);
}
