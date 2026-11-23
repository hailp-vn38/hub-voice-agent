use super::Sounds;

#[test]
fn embedded_sounds_decode_to_mono_at_requested_rate() {
    let sounds = Sounds::load(32_000).unwrap();

    assert_eq!(sounds.rate(), 32_000);
    assert!(sounds.samples(&hub_core::Sound::Ding).unwrap().len() > 32_000 / 2);
    assert!(
        sounds
            .samples(&hub_core::Sound::ReminderChime)
            .unwrap()
            .len()
            > 32_000
    );
}
