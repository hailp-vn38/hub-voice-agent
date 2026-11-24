use anyhow::{Context, Result};
use rubato::{
    Async, FixedAsync, Resampler, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

const INPUT_CHUNK_SAMPLES: usize = 120;

/// Bộ resample sinc mono dùng chung cho input (→16 kHz) và TTS (24 kHz→thiết bị).
pub(super) fn sinc(ratio: f64, chunk_samples: usize) -> Result<Async<f32>> {
    let parameters = SincInterpolationParameters::new(64, WindowFunction::Blackman2)
        .oversampling_factor(64)
        .interpolation(SincInterpolationType::Quadratic);
    Ok(Async::<f32>::new_sinc(
        ratio,
        1.0,
        &parameters,
        chunk_samples,
        1,
        FixedAsync::Input,
    )?)
}

pub(super) struct InputResampler {
    resampler: Async<f32>,
    pending: Vec<f32>,
}

impl InputResampler {
    pub(super) fn new(input_rate: u32) -> Result<Self> {
        let mut resampler = sinc(16_000.0 / f64::from(input_rate), INPUT_CHUNK_SAMPLES)
            .context("không khởi tạo được resampler cho audio input")?;
        let primer = rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(
            vec![0.0; INPUT_CHUNK_SAMPLES],
            1,
            INPUT_CHUNK_SAMPLES,
        )
        .expect("resampler primer has one channel and 120 samples");
        resampler
            .process(&primer, None)
            .expect("configured input resampler accepts its primer");
        Ok(Self {
            resampler,
            pending: Vec::with_capacity(INPUT_CHUNK_SAMPLES),
        })
    }

    pub(super) fn push(&mut self, pcm: &[i16]) -> Vec<i16> {
        self.pending
            .extend(pcm.iter().map(|sample| f32::from(*sample) / 32_768.0));
        let mut consumed = 0;
        let mut output = Vec::new();
        while self.pending.len() - consumed >= INPUT_CHUNK_SAMPLES {
            let input = self.pending[consumed..consumed + INPUT_CHUNK_SAMPLES].to_vec();
            let input = rubato::audioadapter_buffers::owned::InterleavedOwned::new_from(
                input,
                1,
                INPUT_CHUNK_SAMPLES,
            )
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
            consumed += INPUT_CHUNK_SAMPLES;
        }
        let _ = self.pending.drain(..consumed);
        output
    }
}

#[cfg(test)]
#[path = "resample_tests.rs"]
mod tests;
