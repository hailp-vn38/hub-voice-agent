use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, ensure};
use hub_core::Sensitivity;
use ort::{session::Session, value::Tensor};

const MODEL_BLOCK_SAMPLES: usize = 1_280;
const MELSPECTROGRAM_CONTEXT_SAMPLES: usize = 480;
const MEL_BINS: usize = 32;
const MEL_WINDOW_FRAMES: usize = 76;
const EMBEDDING_SIZE: usize = 96;
const CLASSIFIER_WINDOW_FRAMES: usize = 16;
const WAKE_THRESHOLD: f32 = 0.5;
const WAKE_SUPPRESSION: Duration = Duration::from_secs(2);

pub(super) trait WakeDetector {
    fn push(&mut self, pcm: &[i16]) -> bool;
    /// Độ nhạy càng cao thì ngưỡng điểm càng thấp. Mặc định không làm gì.
    fn set_sensitivity(&mut self, _sensitivity: Sensitivity) {}
}

fn threshold(sensitivity: Sensitivity) -> f32 {
    match sensitivity {
        Sensitivity::Low => 0.7,
        Sensitivity::Medium => WAKE_THRESHOLD,
        Sensitivity::High => 0.3,
    }
}

pub(super) fn load_default() -> Option<Box<dyn WakeDetector>> {
    let models_dir = std::env::var_os("AI_HUB_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("models"));
    load_optional(&models_dir)
}

pub(super) fn load_optional(models_dir: &Path) -> Option<Box<dyn WakeDetector>> {
    match OpenWakeWord::load(models_dir) {
        Ok(detector) => Some(Box::new(detector)),
        Err(error) => {
            eprintln!("Wake word disabled: {error:#}");
            None
        }
    }
}

struct OpenWakeWord {
    melspectrogram: Session,
    embedding: Session,
    classifier: Session,
    raw_history: VecDeque<i16>,
    mel_history: VecDeque<[f32; MEL_BINS]>,
    embeddings: VecDeque<[f32; EMBEDDING_SIZE]>,
    debouncer: WakeDebouncer,
    disabled: bool,
}

impl OpenWakeWord {
    fn load(models_dir: &Path) -> Result<Self> {
        let environment = ort::init().with_name("ai-hub-wake-word").build()?;
        let load_session = |filename: &str| -> Result<Session> {
            let path = models_dir.join(filename);
            Session::builder(&environment)
                .map_err(|error| anyhow!(error.to_string()))?
                .with_intra_threads(1)
                .map_err(|error| anyhow!(error.to_string()))?
                .commit_from_file(&path)
                .with_context(|| format!("load {filename}"))
        };

        Ok(Self {
            melspectrogram: load_session("melspectrogram.onnx")?,
            embedding: load_session("embedding_model.onnx")?,
            classifier: load_session("hey_jarvis.onnx")?,
            raw_history: VecDeque::with_capacity(
                MODEL_BLOCK_SAMPLES + MELSPECTROGRAM_CONTEXT_SAMPLES,
            ),
            mel_history: std::iter::repeat_n([1.0; MEL_BINS], MEL_WINDOW_FRAMES).collect(),
            embeddings: VecDeque::with_capacity(CLASSIFIER_WINDOW_FRAMES),
            debouncer: WakeDebouncer::default(),
            disabled: false,
        })
    }

    fn score(&mut self, pcm: &[i16]) -> Result<Option<f32>> {
        ensure!(
            pcm.len() == MODEL_BLOCK_SAMPLES,
            "wake detector expects 80 ms (1280 samples) blocks"
        );

        self.raw_history.extend(pcm.iter().copied());
        while self.raw_history.len() > MODEL_BLOCK_SAMPLES + MELSPECTROGRAM_CONTEXT_SAMPLES {
            self.raw_history.pop_front();
        }

        let samples = self
            .raw_history
            .iter()
            .map(|sample| f32::from(*sample))
            .collect::<Vec<_>>();
        let input = Tensor::from_array(([1_usize, samples.len()], samples.into_boxed_slice()))?;
        let output = self.melspectrogram.run(ort::inputs![input])?;
        let (_, mel_values) = output[0].try_extract_tensor::<f32>()?;
        ensure!(
            !mel_values.is_empty() && mel_values.len() % MEL_BINS == 0,
            "melspectrogram output has an unexpected shape"
        );
        for frame in mel_values.as_chunks::<MEL_BINS>().0 {
            let mut mel = [0.0; MEL_BINS];
            for (value, source) in mel.iter_mut().zip(frame) {
                *value = source / 10.0 + 2.0;
            }
            self.mel_history.push_back(mel);
            if self.mel_history.len() > MEL_WINDOW_FRAMES {
                self.mel_history.pop_front();
            }
        }

        let mel_values = self
            .mel_history
            .iter()
            .flat_map(|frame| frame.iter().copied())
            .collect::<Vec<_>>();
        let input = Tensor::from_array((
            [1_usize, MEL_WINDOW_FRAMES, MEL_BINS, 1],
            mel_values.into_boxed_slice(),
        ))?;
        let output = self.embedding.run(ort::inputs![input])?;
        let (_, embedding) = output[0].try_extract_tensor::<f32>()?;
        ensure!(
            embedding.len() == EMBEDDING_SIZE,
            "embedding model output has an unexpected shape"
        );
        let mut frame = [0.0; EMBEDDING_SIZE];
        frame.copy_from_slice(embedding);
        self.embeddings.push_back(frame);
        if self.embeddings.len() > CLASSIFIER_WINDOW_FRAMES {
            self.embeddings.pop_front();
        }
        if self.embeddings.len() < CLASSIFIER_WINDOW_FRAMES {
            return Ok(None);
        }

        let embedding_values = self
            .embeddings
            .iter()
            .flat_map(|frame| frame.iter().copied())
            .collect::<Vec<_>>();
        let input = Tensor::from_array((
            [1_usize, CLASSIFIER_WINDOW_FRAMES, EMBEDDING_SIZE],
            embedding_values.into_boxed_slice(),
        ))?;
        let output = self.classifier.run(ort::inputs![input])?;
        let (_, scores) = output[0].try_extract_tensor::<f32>()?;
        let score = scores
            .first()
            .copied()
            .context("hey_jarvis model returned no score")?;
        Ok(Some(score))
    }
}

impl WakeDetector for OpenWakeWord {
    fn push(&mut self, pcm: &[i16]) -> bool {
        if self.disabled {
            return false;
        }

        match self.score(pcm) {
            Ok(Some(score)) => self.debouncer.accept(score, Instant::now()),
            Ok(None) => false,
            Err(error) => {
                self.disabled = true;
                eprintln!("Wake word disabled after inference error: {error:#}");
                false
            }
        }
    }

    fn set_sensitivity(&mut self, sensitivity: Sensitivity) {
        self.debouncer.threshold = threshold(sensitivity);
    }
}

struct WakeDebouncer {
    last_wake: Option<Instant>,
    threshold: f32,
}

impl Default for WakeDebouncer {
    fn default() -> Self {
        Self {
            last_wake: None,
            threshold: WAKE_THRESHOLD,
        }
    }
}

impl WakeDebouncer {
    fn accept(&mut self, score: f32, now: Instant) -> bool {
        if score < self.threshold
            || self
                .last_wake
                .is_some_and(|last_wake| now.duration_since(last_wake) < WAKE_SUPPRESSION)
        {
            return false;
        }

        self.last_wake = Some(now);
        true
    }
}

#[cfg(test)]
#[path = "wake_tests.rs"]
mod tests;
