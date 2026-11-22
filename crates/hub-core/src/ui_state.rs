/// Snapshot of everything the UI renders. The UI only reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiState {
    /// Local time as `HH:MM:SS`.
    pub clock: String,
}
