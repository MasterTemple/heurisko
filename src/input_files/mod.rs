use std::path::Path;

use crate::hsk_file::{HskFile, HskResult, Words};

pub mod sbv;
pub mod srt;
pub mod whisper;
pub mod whisperx;
pub mod youtube;

pub trait TranscriptFile: Sized {
    fn read(path: &Path) -> HskResult<Self>;
    fn into_words(self) -> HskResult<Words>;
    fn into_hsk(path: &Path) -> HskResult<HskFile> {
        Ok(HskFile::from_words(Self::read(path)?.into_words()?))
    }
}
