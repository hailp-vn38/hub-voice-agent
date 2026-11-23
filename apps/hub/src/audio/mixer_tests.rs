use hub_core::{AudioCommand, Sound};
use opus::{Application, Channels, Encoder};

use super::{super::sounds::Sounds, Mixer};

const RATE: u32 = 48_000;

#[test]
fn ding_plays_once_then_silence() {
    let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    mixer.apply(AudioCommand::Play(Sound::Ding));

    let mut output = vec![0.0; RATE as usize * 3];
    mixer.pull(&mut output);

    assert!(output[..RATE as usize].iter().any(|sample| *sample != 0.0));
    assert!(
        output[2 * RATE as usize..]
            .iter()
            .all(|sample| *sample == 0.0)
    );
}

#[test]
fn timer_chime_repeats_until_stopped() {
    let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    mixer.apply(AudioCommand::Play(Sound::TimerChime));

    let mut first = vec![0.0; RATE as usize * 2];
    mixer.pull(&mut first);
    assert!(first.iter().any(|sample| *sample != 0.0));

    let mut later = vec![0.0; RATE as usize * 5];
    mixer.pull(&mut later);
    assert!(later.iter().any(|sample| *sample != 0.0));

    mixer.apply(AudioCommand::StopSound(Sound::TimerChime));
    let mut stopped = [1.0; 256];
    mixer.pull(&mut stopped);
    assert!(stopped.iter().all(|sample| *sample == 0.0));
}

#[test]
fn ding_does_not_replace_a_looping_alert_chime() {
    for chime in [Sound::TimerChime, Sound::ReminderChime] {
        let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
        mixer.apply(AudioCommand::Play(chime.clone()));
        mixer.apply(AudioCommand::Play(Sound::Ding));

        let mut output = vec![0.0; RATE as usize * 8];
        mixer.pull(&mut output);

        assert!(
            output[5 * RATE as usize..6 * RATE as usize]
                .iter()
                .any(|sample| *sample != 0.0),
            "{chime:?} should still be audible after Ding finishes"
        );
    }
}

#[test]
fn stopping_an_alert_chime_keeps_the_overlapping_ding_one_shot() {
    for chime in [Sound::TimerChime, Sound::ReminderChime] {
        let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
        mixer.apply(AudioCommand::Play(chime.clone()));
        mixer.apply(AudioCommand::Play(Sound::Ding));
        mixer.apply(AudioCommand::StopSound(chime));

        let mut ding = vec![0.0; RATE as usize / 2];
        mixer.pull(&mut ding);
        assert!(ding.iter().any(|sample| *sample != 0.0));

        let mut after_ding = vec![0.0; RATE as usize * 3];
        mixer.pull(&mut after_ding);
        assert!(
            after_ding[3 * RATE as usize / 2..]
                .iter()
                .all(|sample| *sample == 0.0)
        );
    }
}

#[test]
fn stop_tts_discards_queued_audio() {
    let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    mixer.apply(AudioCommand::PlayTts(opus_frame(0.6)));

    let mut before_stop = [0.0; 1024];
    mixer.pull(&mut before_stop);
    assert!(before_stop.iter().any(|sample| *sample != 0.0));

    mixer.apply(AudioCommand::StopTts);
    let mut after_stop = [1.0; 128];
    mixer.pull(&mut after_stop);
    assert!(after_stop.iter().all(|sample| *sample == 0.0));
}

#[test]
fn level_tracks_tts_energy_and_returns_to_zero_after_stop() {
    let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    mixer.apply(AudioCommand::PlayTts(opus_frame(0.6)));

    let mut output = [0.0; 1024];
    mixer.pull(&mut output);
    assert!(mixer.level() > 0.0);
    assert!(mixer.level() <= 1.0);

    mixer.apply(AudioCommand::StopTts);
    assert_eq!(mixer.level(), 0.0);
}

#[test]
fn chime_is_quieter_while_tts_is_queued() {
    let mut without_tts = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    without_tts.apply(AudioCommand::Play(Sound::TimerChime));
    let mut normal = [0.0; 1024];
    without_tts.pull(&mut normal);

    let mut with_tts = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    with_tts.apply(AudioCommand::Play(Sound::TimerChime));
    with_tts.apply(AudioCommand::PlayTts(opus_frame(0.0)));
    let mut mixed = [0.0; 1024];
    with_tts.pull(&mut mixed);

    assert!(peak(&mixed) < peak(&normal));
}

#[test]
fn empty_mixer_outputs_silence_without_panicking() {
    let mut mixer = Mixer::new(Sounds::load(RATE).unwrap(), RATE);
    mixer.apply(AudioCommand::StopTts);
    mixer.apply(AudioCommand::StopSound(Sound::ReminderChime));
    mixer.apply(AudioCommand::Play(Sound::ToolDone));

    let mut output = [1.0; 16];
    mixer.pull(&mut output);
    assert_eq!(output, [0.0; 16]);
}

fn opus_frame(amplitude: f32) -> voice_client::OpusFrame {
    let mut encoder = Encoder::new(24_000, Channels::Mono, Application::Audio).unwrap();
    let pcm: Vec<i16> = (0..960)
        .map(|index| {
            (amplitude * 16_000.0 * (index as f32 * 440.0 * std::f32::consts::TAU / 24_000.0).sin())
                as i16
        })
        .collect();
    let mut bytes = vec![0; 1500];
    let length = encoder.encode(&pcm, &mut bytes).unwrap();
    bytes.truncate(length);
    voice_client::OpusFrame(bytes)
}

fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|sample| sample.abs())
        .fold(0.0, f32::max)
}
