use super::*;

#[test]
fn hour_temps_span_exactly_the_day_range() {
    let t = hour_temps(22.0, 31.0);
    assert_eq!(t.len(), 24);
    assert_eq!(t.iter().cloned().fold(f32::MIN, f32::max), 31.0);
    assert_eq!(t.iter().cloned().fold(f32::MAX, f32::min), 22.0);
}

#[test]
fn week_range_covers_every_day() {
    let days = [(22.0, 31.0), (20.0, 27.0), (24.0, 35.0)];
    assert_eq!(week_range(&days), (20.0, 35.0));
}

#[test]
fn rain_chance_is_a_fraction() {
    assert!(hour_rain(0.8).iter().all(|r| (0.0..=1.0).contains(r)));
}

#[test]
fn peak_is_the_single_hottest_hour() {
    // 13h và 15h làm tròn cũng bằng 31°: chỉ một điểm được chọn, là giờ sớm nhất.
    let t = hour_temps(22.0, 31.0);
    assert_eq!(t[peak_hour(&t)], 31.0);
    assert_eq!(peak_hour(&[20.0, 25.0, 25.0, 21.0]), 1);
}
