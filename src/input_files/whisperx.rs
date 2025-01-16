use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WhisperXFile {
    pub id: String,
    pub model: String,
    pub compute_type: String,
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
