use hub_core::selector::{Selection, select};

#[test]
fn vietnamese_labels_are_case_and_accent_insensitive_and_report_all_candidates() {
    let labels = ["Trứng luộc", "trung chien", "cà phê"];
    assert_eq!(select(&labels, "CA PHE"), Selection::Found("cà phê"));
    assert_eq!(
        select(&labels, "TRUNG"),
        Selection::Ambiguous(vec!["Trứng luộc", "trung chien"])
    );
    assert_eq!(select(&labels, "tea"), Selection::NotFound);
}
