use super::level;

#[test]
fn level_stays_in_unit_range() {
    for i in 0..2000 {
        let v = level(i as f32 / 30.0);
        assert!((0.0..=1.0).contains(&v), "t={i} v={v}");
    }
}

#[test]
fn level_varies_over_time() {
    let samples: Vec<f32> = (0..60).map(|i| level(i as f32 / 30.0)).collect();
    let min = samples.iter().cloned().fold(1.0, f32::min);
    let max = samples.iter().cloned().fold(0.0, f32::max);
    assert!(max - min > 0.4, "min={min} max={max}");
}
