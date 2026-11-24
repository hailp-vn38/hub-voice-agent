use anyhow::{Context, Result};
use hub_core::{AudioCommand, Sound};

/// Chuông lặp cho tới khi bị dừng; nghe thử và Ding phát một lần.
fn repeats(sound: &Sound) -> bool {
    matches!(
        sound,
        Sound::TimerChime | Sound::ReminderChime | Sound::PingChime
    )
}
use opus::{Channels, Decoder};
use rubato::{Async, Resampler};
use std::{collections::VecDeque, sync::Arc};
use voice_client::OpusFrame;

use super::{resample, sounds::Sounds};

const TTS_RATE: u32 = 24_000;
const MAX_OPUS_SAMPLES: usize = TTS_RATE as usize * 120 / 1000;
const TTS_CHUNK_SAMPLES: usize = 120;
const CHIME_DURING_TTS: f32 = 0.2;
const CHIME_WITHOUT_TTS: f32 = 0.75;

pub(super) struct Mixer {
    sounds: Sounds,
    rate: u32,
    tts_decoder: Decoder,
    tts_resampler: Option<Async<f32>>,
    tts_pending: VecDeque<f32>,
    tts_samples: VecDeque<f32>,
    alert: Option<SoundPlayback>,
    ding: Option<SoundPlayback>,
    level: f32,
    /// Âm lượng chung 0..=1.
    volume: f32,
}

impl Mixer {
    pub(super) fn new(sounds: Sounds, rate: u32) -> Result<Self> {
        debug_assert_eq!(sounds.rate(), rate);
        Ok(Self {
            sounds,
            rate,
            tts_decoder: Decoder::new(TTS_RATE, Channels::Mono)
                .expect("24 kHz mono is a supported Opus decoder format"),
            tts_resampler: make_resampler(rate)?,
            tts_pending: VecDeque::new(),
            tts_samples: VecDeque::new(),
            alert: None,
            ding: None,
            level: 0.0,
            volume: 1.0,
        })
    }

    pub(super) fn apply(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::PlayTts(frame) => self.queue_tts(frame),
            AudioCommand::StopTts => {
                self.tts_pending.clear();
                self.tts_samples.clear();
                if let Some(resampler) = &mut self.tts_resampler {
                    resampler.reset();
                }
                self.level = 0.0;
            }
            AudioCommand::Play(sound) => self.play(sound),
            AudioCommand::StopSound(sound) => self.stop_sound(sound),
            AudioCommand::SetVolume(percent) => self.volume = f32::from(percent.min(100)) / 100.0,
            AudioCommand::Mic { .. } | AudioCommand::SetWakeWord { .. } => {}
        }
    }

    pub(super) fn pull(&mut self, out: &mut [f32]) {
        let mut tts_energy = 0.0;
        let mut tts_count = 0;

        for sample in out.iter_mut() {
            let tts_active = !self.tts_samples.is_empty();
            let tts = self.tts_samples.pop_front().unwrap_or_default();
            let alert = self.alert.as_mut().and_then(SoundPlayback::next_sample);
            if self.alert.as_ref().is_some_and(SoundPlayback::finished) {
                self.alert = None;
            }
            let ding = self.ding.as_mut().and_then(SoundPlayback::next_sample);
            if self.ding.as_ref().is_some_and(SoundPlayback::finished) {
                self.ding = None;
            }

            if tts_active {
                tts_energy += tts * tts;
                tts_count += 1;
            }
            let gain = if tts_active {
                CHIME_DURING_TTS
            } else {
                CHIME_WITHOUT_TTS
            };
            *sample = ((tts + (alert.unwrap_or_default() + ding.unwrap_or_default()) * gain)
                * self.volume)
                .clamp(-1.0, 1.0);
        }

        self.level = if tts_count == 0 {
            0.0
        } else {
            (tts_energy / tts_count as f32).sqrt()
        };
    }

    pub(super) fn level(&self) -> f32 {
        self.level
    }

    fn queue_tts(&mut self, OpusFrame(packet): OpusFrame) {
        let mut decoded = vec![0.0; MAX_OPUS_SAMPLES];
        let Ok(samples) = self.tts_decoder.decode_float(&packet, &mut decoded, false) else {
            return;
        };
        decoded.truncate(samples);
        if self.rate == TTS_RATE {
            self.tts_samples.extend(decoded);
            return;
        }

        self.tts_pending.extend(decoded);
        while self.tts_pending.len() >= TTS_CHUNK_SAMPLES {
            let input_samples = self.tts_pending.drain(..TTS_CHUNK_SAMPLES).collect();
            let Ok(input) = rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(
                input_samples,
                1,
                TTS_CHUNK_SAMPLES,
            ) else {
                continue;
            };
            let Some(resampler) = &mut self.tts_resampler else {
                return;
            };
            if let Ok(converted) = resampler.process(&input, None) {
                self.tts_samples.extend(converted.take_data());
            }
        }
    }

    fn play(&mut self, sound: Sound) {
        let Some(samples) = self.sounds.samples(&sound) else {
            return;
        };
        let repeat = repeats(&sound);
        let playback = SoundPlayback {
            sound,
            samples,
            cursor: 0,
            pause: 0,
            repeat,
            rate: self.rate as usize,
        };
        match playback.sound {
            Sound::Ding | Sound::Preview(_) => self.ding = Some(playback),
            Sound::TimerChime | Sound::ReminderChime | Sound::PingChime => {
                self.alert = Some(playback)
            }
            Sound::ToolDone => {}
        }
    }

    fn stop_sound(&mut self, sound: Sound) {
        match sound {
            Sound::Ding | Sound::Preview(_) => {
                if self
                    .ding
                    .as_ref()
                    .is_some_and(|playing| playing.sound == sound)
                {
                    self.ding = None;
                }
            }
            Sound::TimerChime | Sound::ReminderChime | Sound::PingChime => {
                if self
                    .alert
                    .as_ref()
                    .is_some_and(|playing| playing.sound == sound)
                {
                    self.alert = None;
                }
            }
            Sound::ToolDone => {}
        }
    }
}

fn make_resampler(rate: u32) -> Result<Option<Async<f32>>> {
    if rate == TTS_RATE {
        return Ok(None);
    }
    resample::sinc(f64::from(rate) / f64::from(TTS_RATE), TTS_CHUNK_SAMPLES)
        .with_context(|| format!("không resample được TTS 24 kHz sang {rate} Hz"))
        .map(Some)
}

struct SoundPlayback {
    sound: Sound,
    samples: Arc<[f32]>,
    cursor: usize,
    pause: usize,
    repeat: bool,
    rate: usize,
}

impl SoundPlayback {
    fn next_sample(&mut self) -> Option<f32> {
        if self.samples.is_empty() {
            return None;
        }
        if self.pause > 0 {
            self.pause -= 1;
            return Some(0.0);
        }
        if self.cursor == self.samples.len() {
            if !self.repeat {
                return None;
            }
            self.cursor = 0;
            self.pause = self.rate.saturating_sub(1);
            return Some(0.0);
        }

        let sample = self.samples[self.cursor];
        self.cursor += 1;
        Some(sample)
    }

    fn finished(&self) -> bool {
        self.samples.is_empty() || (!self.repeat && self.cursor == self.samples.len())
    }
}

#[cfg(test)]
#[path = "mixer_tests.rs"]
mod tests;
