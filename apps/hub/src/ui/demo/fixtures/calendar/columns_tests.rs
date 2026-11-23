use super::assign;

#[test]
fn disjoint_blocks_get_full_width() {
    assert_eq!(assign(&[(60, 120), (120, 180)]), vec![(0, 1), (0, 1)]);
}

#[test]
fn overlapping_blocks_split_columns() {
    assert_eq!(assign(&[(60, 180), (90, 150)]), vec![(0, 2), (1, 2)]);
}

#[test]
fn column_is_reused_after_gap_within_cluster() {
    // c reuses b's column; the cluster has 2 columns.
    assert_eq!(assign(&[(60, 180), (90, 120), (130, 170)]), vec![(0, 2), (1, 2), (1, 2)]);
}

#[test]
fn separate_clusters_are_independent() {
    assert_eq!(assign(&[(0, 60), (30, 90), (300, 360)]), vec![(0, 2), (1, 2), (0, 1)]);
}
