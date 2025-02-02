use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hsk_file::Word;

use super::TranscriptFile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhisperXFile {
    // pub id: String,
    // pub model: String,
    // pub compute_type: String,
    pub segments: Vec<WhisperXSegment>,
    pub word_segments: Vec<WhisperXWord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhisperXSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub words: Vec<WhisperXWord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhisperXWord {
    pub word: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub score: Option<f64>,
}

impl TranscriptFile for WhisperXFile {
    fn read(path: &Path) -> crate::hsk_file::HskResult<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    fn into_words(self) -> crate::hsk_file::HskResult<crate::hsk_file::Words> {
        Ok(self
            .word_segments
            .into_iter()
            .map(|word| Word {
                word: word.word,
                start: word.start,
                end: word.end,
            })
            .collect())
    }
}
