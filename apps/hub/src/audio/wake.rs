pub(super) trait WakeDetector {
    fn push(&mut self, pcm: &[i16]) -> bool;
}
