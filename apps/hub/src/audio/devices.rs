use anyhow::{Context, Result, bail, ensure};
use cpal::{
    Device, FromSample, SampleFormat, SizedSample, Stream, StreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use rtrb::Consumer;

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
