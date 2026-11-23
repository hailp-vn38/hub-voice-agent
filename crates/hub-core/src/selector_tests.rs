use super::{Selection, select};

#[test]
fn vietnamese_labels_are_case_and_accent_insensitive_and_report_all_candidates() {
    let labels = ["Trứng luộc", "trung chien", "cà phê"];
    assert_eq!(select(&labels, "CA PHE"), Selection::Found(2));
    assert_eq!(
        select(&labels, "TRUNG"),
        Selection::Ambiguous(vec!["Trứng luộc", "trung chien"])
    );
    assert_eq!(select(&labels, "tea"), Selection::NotFound);
}

#[test]
fn exact_label_wins_over_labels_containing_it() {
    let labels = ["trứng vịt", "Trứng"];
    assert_eq!(select(&labels, "trứng"), Selection::Found(1));
    assert_eq!(select(&labels, "vịt"), Selection::Found(0));
}
