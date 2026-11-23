use super::Presentation;
use hub_core::present::{CountdownItemView, CountdownUnit, CountdownsView};

fn item(key: &str, label: &str) -> CountdownItemView {
    CountdownItemView {
        key: key.into(),
        label: label.into(),
        value: "10".into(),
        unit: CountdownUnit::Days,
        target: "Thứ hai 01/12".into(),
        lunar: String::new(),
        repeat_text: String::new(),
        progress: 0.5,
        progress_text: "50% chặng đường".into(),
    }
}

fn view(main_key: &str, main_label: &str, others: &[(&str, &str)]) -> CountdownsView {
    CountdownsView {
        title: "Đếm ngược · 3 mốc".into(),
        main: item(main_key, main_label),
        has_main: true,
        others: others.iter().map(|(key, label)| item(key, label)).collect(),
        confetti_serial: 0,
    }
}

#[test]
fn selecting_a_countdown_promotes_it_and_keeps_it_selected_across_updates() {
    let mut presentation = Presentation::default();
    presentation.update(&view("1", "Gốc", &[("2", "Đã chọn"), ("3", "Còn lại")]));

    let selected = presentation.select("2").unwrap();
    assert_eq!(selected.main.item.key, "2");
    assert_eq!(selected.main.item.label, "Đã chọn");
    assert_eq!(
        selected
            .others
            .iter()
            .map(|item| item.item.key.as_str())
            .collect::<Vec<_>>(),
        ["1", "3"]
    );

    let updated = presentation.update(&view("1", "Gốc mới", &[("2", "Đã chọn mới")]));
    assert_eq!(updated.main.item.key, "2");
    assert_eq!(updated.main.item.label, "Đã chọn mới");
}

#[test]
fn removing_the_selected_countdown_clears_selection_and_closes_its_detail() {
    let mut presentation = Presentation::default();
    presentation.update(&view("1", "Gốc", &[("2", "Đã chọn")]));
    presentation.select("2").unwrap();
    presentation.open_detail().unwrap();

    let updated = presentation.update(&view("1", "Gốc mới", &[]));

    assert_eq!(updated.main.item.key, "1");
    assert!(!updated.detail_open);
}

#[test]
fn detail_tracks_the_current_main_until_that_countdown_disappears() {
    let mut presentation = Presentation::default();
    presentation.update(&view("1", "Mốc chính", &[("2", "Kế tiếp")]));
    presentation.open_detail().unwrap();

    let updated = presentation.update(&view("2", "Kế tiếp", &[]));

    assert_eq!(updated.main.item.key, "2");
    assert!(!updated.detail_open);
}

#[test]
fn removing_a_main_countdown_fades_before_the_next_main_advances() {
    let mut presentation = Presentation::default();
    presentation.update(&view("1", "Sắp tới", &[("2", "Kế tiếp")]));

    let (fading, id) = presentation.remove("1").unwrap();
    assert_eq!(id, 1);
    assert_eq!(fading.main.item.key, "1");
    assert!(fading.main.fading);

    let after_core_removal = presentation.update(&view("2", "Kế tiếp", &[]));
    assert_eq!(after_core_removal.main.item.key, "1");
    assert!(after_core_removal.main.fading);

    let advanced = presentation.finish_remove("1");
    assert_eq!(advanced.main.item.key, "2");
    assert!(!advanced.main.fading);
}
