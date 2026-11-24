use super::{Capture, MIC_FRAME_SAMPLES, PREROLL_SAMPLES, WAKE_BLOCK_SAMPLES};
use crate::audio::wake::WakeDetector;
use hub_core::{AudioCommand, AudioEvent};
use opus::{Channels, Decoder};

struct FakeWakeDetector {
    detect_on_block: usize,
    blocks_seen: usize,
}

impl WakeDetector for FakeWakeDetector {
    fn push(&mut self, pcm: &[i16]) -> bool {
        assert_eq!(pcm.len(), WAKE_BLOCK_SAMPLES);
        self.blocks_seen += 1;
        self.blocks_seen == self.detect_on_block
    }
}

#[test]
fn wake_detection_uses_complete_80_ms_blocks_without_sending_mic_while_off() {
    let detector = FakeWakeDetector {
        detect_on_block: 1,
        blocks_seen: 0,
    };
    let mut capture = Capture::new(Some(Box::new(detector)));

    let events = capture.push(&vec![0; WAKE_BLOCK_SAMPLES - 1]);
    assert!(events.is_empty());

    let events = capture.push(&[0]);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AudioEvent::WakeWord))
    );
    assert!(
        events
            .iter()
            .all(|event| !matches!(event, AudioEvent::MicFrame(_)))
    );
}

#[test]
fn enabling_mic_sends_decodable_preroll_before_live_60_ms_frames() {
    let detector = FakeWakeDetector {
        detect_on_block: 1,
        blocks_seen: 0,
    };
    let mut capture = Capture::new(Some(Box::new(detector)));

    let wake_events = capture.push(&vec![0; PREROLL_SAMPLES]);
    assert!(
        wake_events
            .iter()
            .any(|event| matches!(event, AudioEvent::WakeWord))
    );
    assert!(
        wake_events
            .iter()
            .all(|event| !matches!(event, AudioEvent::MicFrame(_)))
    );

    capture.apply(&AudioCommand::Mic { on: true });
    let preroll_events = capture.push(&[]);
    let live_events = capture.push(&vec![8_000; MIC_FRAME_SAMPLES]);
    let frames: Vec<_> = preroll_events
        .into_iter()
        .chain(live_events)
        .filter_map(|event| match event {
            AudioEvent::MicFrame(frame) => Some(frame),
            AudioEvent::WakeWord | AudioEvent::TtsPlaybackFinished => None,
        })
        .collect();

    assert_eq!(frames.len(), PREROLL_SAMPLES / MIC_FRAME_SAMPLES + 1);
    let decoded_preroll = decode_frame(&frames[0].0);
    let decoded_live = decode_frame(&frames[frames.len() - 1].0);
    assert_eq!(decoded_preroll.len(), MIC_FRAME_SAMPLES);
    assert_eq!(decoded_live.len(), MIC_FRAME_SAMPLES);
    assert!(rms(&decoded_preroll) < 100.0);
    assert!(rms(&decoded_live) > 1_000.0);
}

#[test]
fn disabling_mic_stops_emitting_mic_frames() {
    let mut capture = Capture::new(None);
    capture.apply(&AudioCommand::Mic { on: true });
    let enabled = capture.push(&vec![3_000; MIC_FRAME_SAMPLES]);
    assert!(
        enabled
            .iter()
            .any(|event| matches!(event, AudioEvent::MicFrame(_)))
    );

    capture.apply(&AudioCommand::Mic { on: false });
    let disabled = capture.push(&vec![3_000; MIC_FRAME_SAMPLES]);
    assert!(
        disabled
            .iter()
            .all(|event| !matches!(event, AudioEvent::MicFrame(_)))
    );
}

#[test]
fn level_tracks_normalized_input_energy_and_silence() {
    let mut capture = Capture::new(None);

    capture.push(&[i16::MAX; 8]);
    assert_eq!(capture.level(), 1.0);

    capture.push(&[0; 8]);
    assert_eq!(capture.level(), 0.0);
}

fn decode_frame(packet: &[u8]) -> Vec<i16> {
    let mut decoder = Decoder::new(16_000, Channels::Mono).unwrap();
    let mut pcm = vec![0; 1_920];
    let samples = decoder.decode(packet, &mut pcm, false).unwrap();
    pcm.truncate(samples);
    pcm
}

fn rms(samples: &[i16]) -> f32 {
    let energy = samples
        .iter()
        .map(|sample| f32::from(*sample) * f32::from(*sample))
        .sum::<f32>();
    (energy / samples.len() as f32).sqrt()
}

#[test]
fn wake_word_can_be_disabled_by_command() {
    let detector = FakeWakeDetector {
        detect_on_block: 1,
        blocks_seen: 0,
    };
    let mut capture = Capture::new(Some(Box::new(detector)));
    capture.apply(&AudioCommand::SetWakeWord {
        enabled: false,
        sensitivity: hub_core::Sensitivity::Medium,
    });
    let events = capture.push(&vec![0; WAKE_BLOCK_SAMPLES]);
    assert!(events.is_empty());
}

#[test]
fn late_wake_detector_gets_the_sensitivity_set_before_it_arrived() {
    use std::sync::{Arc, Mutex};
    struct Recording(Arc<Mutex<Option<hub_core::Sensitivity>>>);
    impl WakeDetector for Recording {
        fn push(&mut self, _pcm: &[i16]) -> bool {
            false
        }
        fn set_sensitivity(&mut self, sensitivity: hub_core::Sensitivity) {
            *self.0.lock().unwrap() = Some(sensitivity);
        }
    }
    let applied = Arc::new(Mutex::new(None));
    let mut capture = Capture::new(None);
    capture.apply(&AudioCommand::SetWakeWord {
        enabled: true,
        sensitivity: hub_core::Sensitivity::High,
    });

    capture.set_wake(Some(Box::new(Recording(applied.clone()))));

    assert_eq!(*applied.lock().unwrap(), Some(hub_core::Sensitivity::High));
}
