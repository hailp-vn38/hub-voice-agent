use super::step;

#[test]
fn step_wraps_both_ways() {
    assert_eq!(step(2, 3, 1), 0);
    assert_eq!(step(0, 3, -1), 2);
    assert_eq!(step(1, 3, 1), 2);
    assert_eq!(step(0, 0, 1), 0);
}
