use crate::{ResponseSpeaker, UiState};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Word {
    pub text: String,
    pub emphasis: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResponseView {
    pub words: Vec<Word>,
    pub speaker: ResponseSpeaker,
    pub line_serial: u64,
    pub fading: bool,
}

pub(super) fn from_state(state: &UiState) -> ResponseView {
    ResponseView {
        words: state
            .voice
            .response_text
            .split_whitespace()
            .map(|text| Word {
                text: text.into(),
                emphasis: false,
            })
            .collect(),
        speaker: state.voice.response_speaker,
        line_serial: state.voice.response_line_serial,
        fading: state.voice.response_fading,
    }
}
