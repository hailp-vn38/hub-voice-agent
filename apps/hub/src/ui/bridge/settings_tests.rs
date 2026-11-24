use super::*;

#[test]
fn choices_map_labels_to_typed_changes() {
    assert_eq!(
        choice("text-size", "Lớn"),
        Some(SettingChange::TextSize(TextSize::Large))
    );
    assert_eq!(
        choice("ambient-after", "Không"),
        Some(SettingChange::AmbientAfter(AmbientAfter::Never))
    );
    assert_eq!(
        choice("sensitivity", "Cao"),
        Some(SettingChange::Sensitivity(Sensitivity::High))
    );
    assert_eq!(
        choice("auto-brightness", "Theo giờ"),
        Some(SettingChange::AutoBrightness(true))
    );
    assert_eq!(
        choice("timezone", "Asia/Ho_Chi_Minh"),
        Some(SettingChange::Timezone("Asia/Ho_Chi_Minh".into()))
    );
}

#[test]
fn unknown_keys_and_labels_are_ignored() {
    assert_eq!(choice("text-size", "Khổng lồ"), None);
    assert_eq!(choice("bogus", "Lớn"), None);
    assert_eq!(toggle("entity:Đèn", true), None);
    assert_eq!(
        toggle("mic-muted", true),
        Some(SettingChange::MicMuted(true))
    );
}

#[test]
fn every_label_round_trips() {
    for (text, size) in TEXT_SIZES {
        assert_eq!(label(&TEXT_SIZES, &size), text);
    }
    for (text, after) in AMBIENT_AFTER {
        assert_eq!(label(&AMBIENT_AFTER, &after), text);
    }
    for (text, level) in SENSITIVITY {
        assert_eq!(label(&SENSITIVITY, &level), text);
    }
}

#[test]
fn text_sizes_scale_the_theme() {
    assert_eq!(text_scale(TextSize::Small), 0.85);
    assert_eq!(text_scale(TextSize::Medium), 1.0);
    assert_eq!(text_scale(TextSize::Large), 1.25);
}
