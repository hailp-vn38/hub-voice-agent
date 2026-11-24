use anyhow::{Context, Result, bail, ensure};
use cpal::{
    Device, FromSample, Sample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::{Consumer, Producer};

pub(super) fn open_output(consumer: Consumer<f32>) -> Result<(Stream, u32)> {
    let host = cpal::default_host();
    let device = output_device(&host)?;
    let supported = device
        .default_output_config()
        .context("không đọc được cấu hình audio output")?;
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

pub(super) fn open_input(producer: Producer<i16>) -> Result<(Stream, u32)> {
    let host = cpal::default_host();
    let device = input_device(&host)?;
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
        .context("không đọc được cấu hình audio input")?;
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

#[cfg(feature = "pi")]
fn output_device(host: &cpal::Host) -> Result<Device> {
    host.output_devices()
        .context("không liệt kê được thiết bị audio output trên Pi")?
        .find(|device| {
            device
                .description()
                .is_ok_and(|description| is_xvf3800_device_name(description.name()))
        })
        .context("Pi yêu cầu thiết bị audio output XVF3800; không tìm thấy thiết bị này")
}

#[cfg(not(feature = "pi"))]
fn output_device(host: &cpal::Host) -> Result<Device> {
    host.default_output_device()
        .context("không tìm thấy thiết bị audio output mặc định")
}

#[cfg(feature = "pi")]
fn input_device(host: &cpal::Host) -> Result<Device> {
    host.input_devices()
        .context("không liệt kê được thiết bị audio input trên Pi")?
        .find(|device| {
            device
                .description()
                .is_ok_and(|description| is_xvf3800_device_name(description.name()))
        })
        .context("Pi yêu cầu thiết bị audio input XVF3800; không tìm thấy thiết bị này")
}

#[cfg(not(feature = "pi"))]
fn input_device(host: &cpal::Host) -> Result<Device> {
    host.default_input_device()
        .context("không tìm thấy thiết bị audio input mặc định")
}

#[cfg(any(feature = "pi", test))]
fn is_xvf3800_device_name(name: &str) -> bool {
    name.to_ascii_lowercase().contains("xvf3800")
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

#[cfg(test)]
#[path = "devices_tests.rs"]
mod tests;
