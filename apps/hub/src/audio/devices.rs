use anyhow::{Context, Result, bail, ensure};
use cpal::{
    Device, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Consumer, Producer};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

pub(super) fn open_default_output(consumer: Consumer<f32>) -> Result<(Stream, u32)> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .context("không tìm thấy thiết bị audio output mặc định")?;
    let supported = device
        .default_output_config()
        .context("không đọc được cấu hình audio output mặc định")?;
    let config = supported.config();
    let channels = usize::from(config.channels);
    ensure!(channels > 0, "thiết bị audio output không có kênh");
    let rate = config.sample_rate;
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, channels, consumer)?,
        SampleFormat::F64 => build::<f64>(&device, config, channels, consumer)?,
        SampleFormat::I8 => build::<i8>(&device, config, channels, consumer)?,
        SampleFormat::I16 => build::<i16>(&device, config, channels, consumer)?,
        SampleFormat::I24 => build::<cpal::I24>(&device, config, channels, consumer)?,
        SampleFormat::I32 => build::<i32>(&device, config, channels, consumer)?,
        SampleFormat::I64 => build::<i64>(&device, config, channels, consumer)?,
        SampleFormat::U8 => build::<u8>(&device, config, channels, consumer)?,
        SampleFormat::U16 => build::<u16>(&device, config, channels, consumer)?,
        SampleFormat::U24 => build::<cpal::U24>(&device, config, channels, consumer)?,
        SampleFormat::U32 => build::<u32>(&device, config, channels, consumer)?,
        SampleFormat::U64 => build::<u64>(&device, config, channels, consumer)?,
        format => bail!("định dạng audio output không được hỗ trợ: {format}"),
    };
    Ok((stream, rate))
}

pub(super) fn open_default_input(producer: Producer<i16>) -> Result<(Stream, u32)> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("không tìm thấy thiết bị audio input mặc định")?;
    let config = device
        .supported_input_configs()
        .ok()
        .and_then(|configs| {
            configs.into_iter().find_map(|range| {
                (range.channels() == 1
                    && range.min_sample_rate() <= 16_000
                    && range.max_sample_rate() >= 16_000
                    && supported_sample_format(range.sample_format()))
                .then(|| range.with_sample_rate(16_000))
            })
        })
        .map(Ok)
        .unwrap_or_else(|| device.default_input_config())
        .context("không đọc được cấu hình audio input mặc định")?;
    let stream_config = config.config();
    let channels = usize::from(stream_config.channels);
    ensure!(channels > 0, "thiết bị audio input không có kênh");
    let rate = stream_config.sample_rate;
    ensure!(rate > 0, "thiết bị audio input không có tần số lấy mẫu");
    let stream = match config.sample_format() {
        SampleFormat::F32 => build_input::<f32>(&device, stream_config, channels, producer)?,
        SampleFormat::F64 => build_input::<f64>(&device, stream_config, channels, producer)?,
        SampleFormat::I8 => build_input::<i8>(&device, stream_config, channels, producer)?,
        SampleFormat::I16 => build_input::<i16>(&device, stream_config, channels, producer)?,
        SampleFormat::I24 => build_input::<cpal::I24>(&device, stream_config, channels, producer)?,
        SampleFormat::I32 => build_input::<i32>(&device, stream_config, channels, producer)?,
        SampleFormat::I64 => build_input::<i64>(&device, stream_config, channels, producer)?,
        SampleFormat::U8 => build_input::<u8>(&device, stream_config, channels, producer)?,
        SampleFormat::U16 => build_input::<u16>(&device, stream_config, channels, producer)?,
        SampleFormat::U24 => build_input::<cpal::U24>(&device, stream_config, channels, producer)?,
        SampleFormat::U32 => build_input::<u32>(&device, stream_config, channels, producer)?,
        SampleFormat::U64 => build_input::<u64>(&device, stream_config, channels, producer)?,
        format => bail!("định dạng audio input không được hỗ trợ: {format}"),
    };
    Ok((stream, rate))
}

pub(super) fn start(stream: &Stream) -> Result<()> {
    stream.play().context("khởi động cpal output stream")
}

fn build<T>(
    device: &Device,
    config: StreamConfig,
    channels: usize,
    mut consumer: Consumer<f32>,
) -> Result<Stream>
where
    T: SizedSample + FromSample<f32>,
{
    Ok(device.build_output_stream(
        config,
        move |output: &mut [T], _| {
            let mut frame = 0.0;
            for (index, sample) in output.iter_mut().enumerate() {
                if index % channels == 0 {
                    frame = consumer.pop().unwrap_or(0.0);
                }
                *sample = T::from_sample(frame);
            }
        },
        |error| eprintln!("Lỗi audio output: {error}"),
        None,
    )?)
}

fn build_input<T>(
    device: &Device,
    config: StreamConfig,
    channels: usize,
    mut producer: Producer<i16>,
) -> Result<Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    Ok(device.build_input_stream(
        config,
        move |input: &[T], _| {
            for frame in input.chunks_exact(channels) {
                let sample = frame
                    .iter()
                    .map(|sample| f32::from_sample(*sample))
                    .sum::<f32>()
                    / channels as f32;
                let sample = (sample.clamp(-1.0, 1.0) * 32_768.0) as i16;
                let _ = producer.push(sample);
            }
        },
        |error| eprintln!("Lỗi audio input: {error}"),
        None,
    )?)
}

fn supported_sample_format(format: SampleFormat) -> bool {
    matches!(
        format,
        SampleFormat::F32
            | SampleFormat::F64
            | SampleFormat::I8
            | SampleFormat::I16
            | SampleFormat::I24
            | SampleFormat::I32
            | SampleFormat::I64
            | SampleFormat::U8
            | SampleFormat::U16
            | SampleFormat::U24
            | SampleFormat::U32
            | SampleFormat::U64
    )
}

pub(super) struct InputResampler {
    resampler: Async<f32>,
    pending: Vec<f32>,
}

impl InputResampler {
    pub(super) fn new(input_rate: u32) -> Result<Self> {
        let parameters = SincInterpolationParameters::new(64, WindowFunction::Blackman2)
            .oversampling_factor(64)
            .interpolation(SincInterpolationType::Quadratic);
        let mut resampler = Async::<f32>::new_sinc(
            16_000.0 / f64::from(input_rate),
            1.0,
            &parameters,
            120,
            1,
            FixedAsync::Input,
        )
        .context("không khởi tạo được resampler cho audio input")?;
        let primer =
            rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(vec![0.0; 120], 1, 120)
                .expect("resampler primer has one channel and 120 samples");
        resampler
            .process(&primer, None)
            .expect("configured input resampler accepts its primer");
        Ok(Self {
            resampler,
            pending: Vec::with_capacity(120),
        })
    }

    pub(super) fn push(&mut self, pcm: &[i16]) -> Vec<i16> {
        self.pending
            .extend(pcm.iter().map(|sample| f32::from(*sample) / 32_768.0));
        let mut consumed = 0;
        let mut output = Vec::new();
        while self.pending.len() - consumed >= 120 {
            let input = self.pending[consumed..consumed + 120].to_vec();
            let input =
                rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(input, 1, 120)
                    .expect("resampler input has one channel and 120 samples");
            let converted = self
                .resampler
                .process(&input, None)
                .expect("configured input resampler accepts 120 samples");
            output.extend(
                converted
                    .take_data()
                    .into_iter()
                    .map(|sample| (sample * 32_768.0).clamp(-32_768.0, 32_767.0) as i16),
            );
            consumed += 120;
        }
        let _ = self.pending.drain(..consumed);
        output
    }
}

#[cfg(test)]
#[path = "devices_tests.rs"]
mod tests;
