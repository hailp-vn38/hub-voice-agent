use std::collections::VecDeque;

use hub_core::{AudioCommand, AudioEvent, Sensitivity};
use opus::{Application, Channels, Encoder};
use voice_client::OpusFrame;

use super::wake::WakeDetector;

const SAMPLE_RATE: u32 = 16_000;
const MIC_FRAME_SAMPLES: usize = SAMPLE_RATE as usize * 60 / 1_000;
const WAKE_BLOCK_SAMPLES: usize = SAMPLE_RATE as usize * 80 / 1_000;
const PREROLL_SAMPLES: usize = SAMPLE_RATE as usize * 1_500 / 1_000;
const MAX_OPUS_PACKET_BYTES: usize = 4_000;

pub(super) struct Capture {
    wake: Option<Box<dyn WakeDetector>>,
    wake_block: [i16; WAKE_BLOCK_SAMPLES],
    wake_block_len: usize,
    preroll: VecDeque<i16>,
    mic_frame: Vec<i16>,
    queued_events: Vec<AudioEvent>,
    encoder: Encoder,
    mic_on: bool,
    wake_enabled: bool,
    sensitivity: Sensitivity,
    level: f32,
}

impl Capture {
    pub(super) fn new(wake: Option<Box<dyn WakeDetector>>) -> Self {
        Self {
            wake,
            wake_block: [0; WAKE_BLOCK_SAMPLES],
            wake_block_len: 0,
            preroll: VecDeque::with_capacity(PREROLL_SAMPLES),
            mic_frame: Vec::with_capacity(MIC_FRAME_SAMPLES),
            queued_events: Vec::new(),
            encoder: Encoder::new(SAMPLE_RATE, Channels::Mono, Application::Voip)
                .expect("16 kHz mono Opus is supported"),
            mic_on: false,
            wake_enabled: true,
            sensitivity: Sensitivity::default(),
            level: 0.0,
        }
    }

    pub(super) fn apply(&mut self, command: &AudioCommand) {
        match command {
            AudioCommand::Mic { on: true } if !self.mic_on => {
                self.queue_preroll();
                self.mic_on = true;
            }
            AudioCommand::Mic { on: false } => {
                self.mic_on = false;
                self.mic_frame.clear();
                self.preroll.clear();
                self.queued_events.clear();
            }
            AudioCommand::SetWakeWord {
                enabled,
                sensitivity,
            } => {
                self.wake_enabled = *enabled;
                self.sensitivity = *sensitivity;
                if let Some(detector) = &mut self.wake {
                    detector.set_sensitivity(*sensitivity);
                }
            }
            _ => {}
        }
    }

    /// Gắn detector được nạp muộn (model vừa tải xong), giữ độ nhạy đã đặt.
    pub(super) fn set_wake(&mut self, mut wake: Option<Box<dyn WakeDetector>>) {
        if let Some(detector) = &mut wake {
            detector.set_sensitivity(self.sensitivity);
        }
        self.wake = wake;
    }

    pub(super) fn push(&mut self, pcm: &[i16]) -> Vec<AudioEvent> {
        if !pcm.is_empty() {
            self.level = level(pcm);
        }
        let mut events = std::mem::take(&mut self.queued_events);

        for &sample in pcm {
            if self.preroll.len() == PREROLL_SAMPLES {
                self.preroll.pop_front();
            }
            self.preroll.push_back(sample);

            self.wake_block[self.wake_block_len] = sample;
            self.wake_block_len += 1;
            if self.wake_block_len == WAKE_BLOCK_SAMPLES {
                if self.wake_enabled
                    && self
                        .wake
                        .as_mut()
                        .is_some_and(|detector| detector.push(&self.wake_block))
                {
                    // Bỏ âm thanh chứa wake word để preroll chỉ còn lời nói sau nó.
                    self.preroll.clear();
                    events.push(AudioEvent::WakeWord);
                }
                self.wake_block_len = 0;
            }

            if self.mic_on {
                self.mic_frame.push(sample);
                if self.mic_frame.len() == MIC_FRAME_SAMPLES {
                    let packet = encode(&mut self.encoder, &self.mic_frame);
                    events.push(AudioEvent::MicFrame(packet));
                    self.mic_frame.clear();
                }
            }
        }

        events
    }

    pub(super) fn level(&self) -> f32 {
        self.level
    }

    fn queue_preroll(&mut self) {
        let Self {
            preroll,
            queued_events,
            encoder,
            ..
        } = self;
        let skip = preroll.len() % MIC_FRAME_SAMPLES;
        for frame in preroll.make_contiguous()[skip..]
            .as_chunks::<MIC_FRAME_SAMPLES>()
            .0
        {
            queued_events.push(AudioEvent::MicFrame(encode(encoder, frame)));
        }
    }
}

fn encode(encoder: &mut Encoder, pcm: &[i16]) -> OpusFrame {
    let mut packet = vec![0; MAX_OPUS_PACKET_BYTES];
    let length = encoder
        .encode(pcm, &mut packet)
        .expect("a 60 ms 16 kHz mono frame is supported by Opus");
    packet.truncate(length);
    OpusFrame(packet)
}

fn level(pcm: &[i16]) -> f32 {
    if pcm.is_empty() {
        return 0.0;
    }
    let energy = pcm
        .iter()
        .map(|sample| f64::from(*sample) * f64::from(*sample))
        .sum::<f64>();
    ((energy / pcm.len() as f64).sqrt() / f64::from(i16::MAX)).clamp(0.0, 1.0) as f32
}

#[cfg(test)]
#[path = "capture_tests.rs"]
mod tests;
