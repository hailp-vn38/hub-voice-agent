//! Mức âm cho Orb: RMS 0..1 → thang dB 0..1 mà Orb hiển thị được.

/// Dưới mức này coi là im lặng (nhiễu phòng); từ mức trần trở lên là to nhất.
const FLOOR_DB: f32 = -50.0;
const CEILING_DB: f32 = -10.0;
/// Thay đổi nhỏ hơn bước này không đáng gửi lên UI.
const MIN_STEP: f32 = 0.01;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Levels {
    pub mic: f32,
    pub tts: f32,
}

impl Levels {
    pub(super) fn from_rms(mic: f32, tts: f32) -> Self {
        Self {
            mic: meter(mic),
            tts: meter(tts),
        }
    }

    pub(super) fn differs_from(&self, other: &Self) -> bool {
        (self.mic - other.mic).abs() >= MIN_STEP || (self.tts - other.tts).abs() >= MIN_STEP
    }
}

fn meter(rms: f32) -> f32 {
    if rms <= 0.0 {
        return 0.0;
    }
    ((20.0 * rms.log10() - FLOOR_DB) / (CEILING_DB - FLOOR_DB)).clamp(0.0, 1.0)
}

#[cfg(test)]
#[path = "levels_tests.rs"]
mod tests;
