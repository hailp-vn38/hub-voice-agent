use super::Levels;

#[test]
fn silence_and_room_noise_keep_the_orb_still() {
    assert_eq!(Levels::from_rms(0.0, 0.0), Levels::default());
    assert_eq!(Levels::from_rms(0.001, 0.0).mic, 0.0);
}

#[test]
fn normal_speech_moves_the_orb_through_most_of_its_range() {
    let quiet = Levels::from_rms(0.02, 0.0).mic;
    let loud = Levels::from_rms(0.1, 0.0).mic;

    assert!(quiet > 0.3, "quiet speech {quiet}");
    assert!(loud > quiet && loud < 1.0, "loud speech {loud}");
    assert_eq!(Levels::from_rms(1.0, 1.0), Levels { mic: 1.0, tts: 1.0 });
}

#[test]
fn tiny_changes_are_not_worth_publishing() {
    let shown = Levels { mic: 0.5, tts: 0.0 };

    assert!(
        !Levels {
            mic: 0.505,
            tts: 0.0
        }
        .differs_from(&shown)
    );
    assert!(
        Levels {
            mic: 0.52,
            tts: 0.0
        }
        .differs_from(&shown)
    );
}
