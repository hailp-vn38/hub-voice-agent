use super::{dec_days, dec_hms};

#[test]
fn live_value_counts_down_one_second() {
    assert_eq!(dec_hms("05:42:10"), "05:42:09");
    assert_eq!(dec_hms("05:42:00"), "05:41:59");
    assert_eq!(dec_hms("01:00:00"), "00:59:59");
    assert_eq!(dec_hms("00:00:00"), "00:00:00");
}

#[test]
fn days_value_drops_by_one() {
    assert_eq!(dec_days("75"), "74");
    assert_eq!(dec_days("1"), "0");
    assert_eq!(dec_days("0"), "0");
}
