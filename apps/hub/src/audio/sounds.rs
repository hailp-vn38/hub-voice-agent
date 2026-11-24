use anyhow::{Context, Result, bail};
use hub_core::{Sound, Tone};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};
use std::io::{Cursor, ErrorKind};
use std::sync::Arc;
use symphonia::core::{
    audio::SampleBuffer, codecs::DecoderOptions, errors::Error as SymphoniaError,
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};

const PING: &[u8] = include_bytes!("../../assets/audio/ping.mp3");
const REMINDER: &[u8] = include_bytes!("../../assets/audio/reminder.mp3");

pub(super) struct Sounds {
    ding: Arc<[f32]>,
    reminder: Arc<[f32]>,
    rate: u32,
}

impl Sounds {
    pub(super) fn load(rate: u32) -> Result<Self> {
        let (ding, ding_rate) = decode_mono(PING).context("giải mã ping.mp3")?;
        let (reminder, reminder_rate) = decode_mono(REMINDER).context("giải mã reminder.mp3")?;

        Ok(Self {
            ding: resample(&ding, ding_rate, rate)?.into(),
            reminder: resample(&reminder, reminder_rate, rate)?.into(),
            rate,
        })
    }

    pub(super) fn samples(&self, sound: &Sound) -> Option<Arc<[f32]>> {
        match sound {
            Sound::Ding | Sound::PingChime | Sound::Preview(Tone::Ping) => {
                Some(Arc::clone(&self.ding))
            }
            Sound::TimerChime | Sound::ReminderChime | Sound::Preview(Tone::Reminder) => {
                Some(Arc::clone(&self.reminder))
            }
            Sound::ToolDone => None,
        }
    }

    pub(super) fn rate(&self) -> u32 {
        self.rate
    }
}

fn decode_mono(bytes: &'static [u8]) -> Result<(Vec<f32>, u32)> {
    let source = MediaSourceStream::new(Box::new(Cursor::new(bytes)), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let probed = symphonia::default::get_probe().format(
        &hint,
        source,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;
    let track = format.default_track().context("không có audio track")?;
    let track_id = track.id;
    let mut decoder =
        symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default())?;
    let mut output = Vec::new();
    let mut rate = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error)) if error.kind() == ErrorKind::UnexpectedEof => {
                break;
            }
            Err(error) => return Err(error.into()),
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(error.into()),
        };
        rate = decoded.spec().rate;
        let channels = decoded.spec().channels.count();
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        buffer.copy_interleaved_ref(decoded);
        output.extend(
            buffer
                .samples()
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32),
        );
    }

    if output.is_empty() || rate == 0 {
        bail!("MP3 không có mẫu âm thanh giải mã được");
    }
    Ok((output, rate))
}

pub(super) fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Result<Vec<f32>> {
    if samples.is_empty() || source_rate == target_rate {
        return Ok(samples.to_vec());
    }
    if source_rate == 0 || target_rate == 0 {
        bail!("tần số lấy mẫu phải lớn hơn 0");
    }

    let input = rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(
        samples.to_vec(),
        1,
        samples.len(),
    )?;
    let parameters = SincInterpolationParameters::new(64, WindowFunction::Blackman2)
        .oversampling_factor(64)
        .interpolation(SincInterpolationType::Quadratic);
    let mut resampler = Async::<f32>::new_sinc(
        target_rate as f64 / source_rate as f64,
        1.0,
        &parameters,
        1024,
        1,
        FixedAsync::Input,
    )?;
    Ok(resampler
        .process_all(&input, samples.len(), None)?
        .take_data())
}

#[cfg(test)]
#[path = "sounds_tests.rs"]
mod tests;
