use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zstd::stream::{read::Decoder, write::Encoder};

use crate::input_files::sbv::SbvFile;
use crate::input_files::srt::SrtFile;
use crate::input_files::whisper::UnalignedWhisperXFile;
use crate::input_files::whisperx::WhisperXFile;
use crate::input_files::youtube::YouTubeTranscript;
use crate::searcher::{normalize_word, Map};

pub type HskResult<T> = Result<T, Box<dyn Error>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Word {
    pub word: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HskFile {
    pub words: Vec<Word>,
    pub word_index_map: WordIndexMap,
}

#[allow(unused)]
impl HskFile {
    pub fn convert(source: &Path, dest: &Path) -> HskResult<()> {
        let hsk = HskFile::infer(&source)?;
        hsk.save(dest)
    }

    pub fn infer(path: &Path) -> HskResult<Self> {
        if path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(file) = HskFile::from_whisperx(&path) {
                Ok(file)
            } else if let Ok(file) = HskFile::from_youtube(&path) {
                Ok(file)
            } else {
                HskFile::from_whisper(&path)
            }
        } else {
            if let Ok(file) = HskFile::from_srt(&path) {
                Ok(file)
            } else {
                HskFile::from_sbv(&path)
            }
            // if let Ok(file) = HskFile::from_sbv(&path) {
            //     Ok(file)
            // }
        }
    }

    pub fn from_words(words: Vec<Word>) -> Self {
        Self {
            word_index_map: index_words(&words),
            words,
        }
    }

    pub fn from_whisper(path: &Path) -> HskResult<Self> {
        let contents = fs::read_to_string(path)?;
        let whisper_file: UnalignedWhisperXFile = serde_json::from_str(&contents)?;
        let words = whisper_file
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
            .collect();
        Ok(Self::from_words(words))
    }

    pub fn from_youtube(path: &Path) -> HskResult<Self> {
        let contents = fs::read_to_string(path)?;
        let youtube_transcript: Vec<YouTubeTranscript> = serde_json::from_str(&contents)?;
        let mut words = youtube_transcript
            // .as_slice()
            .windows(2)
            .flat_map(|el| {
                let this = &el[0];
                let next = &el[1];
                this.text.split_whitespace().map(|word| Word {
                    word: word.to_string(),
                    start: Some(this.start),
                    end: Some(next.start),
                })
            })
            .collect::<Vec<_>>();
        if let Some(last) = youtube_transcript.last() {
            words.extend(last.text.split_whitespace().map(|word| Word {
                word: word.to_string(),
                start: Some(last.start),
                end: None,
            }));
        }
        Ok(Self::from_words(words))
    }

    pub fn from_whisperx(path: &Path) -> HskResult<Self> {
        let contents = fs::read_to_string(path)?;
        let whisperx_file: WhisperXFile = serde_json::from_str(&contents)?;
        let words = whisperx_file
            .word_segments
            .into_iter()
            .map(|word| Word {
                word: word.word,
                start: word.start,
                end: word.end,
            })
            .collect();
        Ok(Self::from_words(words))
    }

    pub fn from_srt(path: &Path) -> HskResult<Self> {
        let file = SrtFile::from_file(path)?;
        let words = file
            .segments
            .into_iter()
            .flat_map(|seg| {
                seg.text
                    .split_whitespace()
                    .map(|word| Word {
                        word: word.to_string(),
                        start: Some(seg.start.in_seconds()),
                        end: Some(seg.end.in_seconds()),
                    })
                    // compiler gets mad if I don't collect :(
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Self::from_words(words))
    }

    pub fn from_sbv(path: &Path) -> HskResult<Self> {
        let file = SbvFile::from_file(path)?;
        let words = file
            .segments
            .into_iter()
            .flat_map(|seg| {
                seg.text
                    .split_whitespace()
                    .map(|word| Word {
                        word: word.to_string(),
                        start: Some(seg.start.in_seconds()),
                        end: Some(seg.end.in_seconds()),
                    })
                    // compiler gets mad if I don't collect :(
                    .collect::<Vec<_>>()
            })
            .collect();
        Ok(Self::from_words(words))
    }

    pub fn save(&self, path: &Path) -> HskResult<()> {
        let data = serde_json::to_string(self)?.into_bytes();
        compress_and_write(data, path)
    }

    pub fn read(path: &Path) -> HskResult<Self> {
        let data = read_and_decompress(path)?;
        let value = serde_json::from_str(&String::from_utf8(data)?)?;
        Ok(value)
    }
}

const COMPRESSION_LEVEL: i32 = 3;

fn compress_and_write(data: Vec<u8>, path: &Path) -> HskResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    let mut encoder = Encoder::new(file, COMPRESSION_LEVEL)?;
    encoder.write_all(&data)?;
    encoder.finish()?;
    Ok(())
}

fn read_and_decompress(path: &Path) -> HskResult<Vec<u8>> {
    let file = File::open(path)?;
    let mut decoder = Decoder::new(file)?;
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    Ok(decompressed_data)
}

pub type WordIndexMap = Map<String, Vec<usize>>;
fn index_words(words: &Vec<Word>) -> WordIndexMap {
    let index_word_pairs = words
        .iter()
        .enumerate()
        .map(|(idx, word)| (idx, normalize_word(&word.word)))
        .collect::<Vec<_>>();

    let mut word_index_map = WordIndexMap::default();
    for (idx, word) in index_word_pairs {
        if let Some(existing_entry) = word_index_map.get_mut(&word) {
            existing_entry.push(idx);
        } else {
            _ = word_index_map.insert(word, vec![idx]);
        }
    }
    word_index_map
}

// saving in case i want to move the index map out of the HskFile again
// pub struct IndexedHskFile {
//     pub words: Vec<Word>,
//     pub word_index_map: WordIndexMap,
// }
// impl From<HskFile> for IndexedHskFile {
//     fn from(value: HskFile) -> Self {
//         let HskFile { words } = value;
//         Self {
//             word_index_map: index_words(&words),
//             words,
//         }
//     }
// }
