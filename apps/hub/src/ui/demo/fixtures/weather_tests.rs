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
