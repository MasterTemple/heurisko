use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hsk_file::Word;

use super::TranscriptFile;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnalignedWhisperXFile {
    // pub id: String,
    // pub model: String,
    // pub compute_type: String,
    pub segments: Vec<UnalignedWhisperXSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnalignedWhisperXSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

impl TranscriptFile for UnalignedWhisperXFile {
    fn read(path: &Path) -> crate::hsk_file::HskResult<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    fn into_words(self) -> crate::hsk_file::HskResult<crate::hsk_file::Words> {
        Ok(self
            .segments
            .into_iter()
            .flat_map(|seg| {
                seg.text
                    .split_whitespace()
                    .map(|word| Word {
                        word: word.to_string(),
                        start: Some(seg.start),
                        end: Some(seg.end),
                    })
                    // compiler gets mad if I don't collect :(
                    .collect::<Vec<_>>()
            })
            .collect())
    }
}
