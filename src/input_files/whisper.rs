use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnalignedWhisperXFile {
    pub id: String,
    pub model: String,
    pub compute_type: String,
    pub segments: Vec<UnalignedWhisperXSegment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnalignedWhisperXSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}
