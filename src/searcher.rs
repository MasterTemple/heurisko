use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    hsk_file::{HskFile, Word},
    merge::{merge_special, WordSegmentRange},
    CONFIG,
};

pub type Map<K, V> = BTreeMap<K, V>;

pub fn normalize_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

pub type WordIndices = Vec<usize>;
pub type TranscriptId = usize;
pub type TranscriptWordIndices = (TranscriptId, WordIndices);

pub type WordToWordIndices = Map<String, WordIndices>;

pub type WordToTranscriptAndWordIndicesMap = Map<String, Vec<TranscriptWordIndices>>;
pub type OrganizedSearchResult = Map<usize, Map<usize, Vec<QueryResult>>>;
// pub type OrganizedSearchResult = Map<usize, Map<usize, Vec<SearchResult>>>;

pub struct Searcher {
    // pub transcript_paths: Map<TranscriptId, PathBuf>,
    // pub transcript_paths: Vec<PathBuf>,
    // pub transcript_paths: Vec<Cow<'static, str>>,
    pub transcript_paths: Vec<String>,
    // transcript id -> Word
    pub transcript_words: Map<TranscriptId, Vec<Word>>,
    // word to transcript id
    pub map: WordToTranscriptAndWordIndicesMap,
}

impl Searcher {
    pub fn load() -> Self {
        let data_dir = CONFIG.data_dir();
        let mut transcript_id: usize = 0;
        let mut transcript_paths = Vec::new();
        let mut transcript_words: Map<TranscriptId, Vec<Word>> = Map::new();
        let mut map: WordToTranscriptAndWordIndicesMap = Map::new();
        for entry in WalkDir::new(&data_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "hsk") {
                if let Ok(file) = HskFile::read(path) {
                    let mut relative_path = path.strip_prefix(&data_dir).unwrap().to_path_buf();
                    relative_path.set_extension("");
                    transcript_paths.push(relative_path.to_string_lossy().to_string());
                    transcript_words.insert(transcript_id, file.words);
                    file.word_index_map.into_iter().for_each(|(word, indices)| {
                        if let Some(existing_entry) = map.get_mut(&word) {
                            existing_entry.push((transcript_id, indices));
                        } else {
                            _ = map.insert(word.clone(), vec![(transcript_id, indices)]);
                        }
                    });
                    transcript_id += 1;
                }
            }
        }
        Self {
            transcript_paths,
            transcript_words,
            map,
        }
    }

    fn word_indices_group_by_transcript(
        &self,
        words: &Vec<String>,
    ) -> Map<TranscriptId, Vec<WordIndices>> {
        let words: Vec<&String> = words.iter().filter(|w| w.len() > 0).collect();
        let mut transcript_to_indices: Map<TranscriptId, Vec<WordIndices>> = Map::default();

        for findings in words.into_iter().filter_map(|word| self.map.get(word)) {
            for finding in findings {
                if let Some(existing_entry) = transcript_to_indices.get_mut(&finding.0) {
                    // existing_entry.extend(finding.1);
                    existing_entry.push(finding.1.clone());
                } else {
                    // transcript_to_indices.insert(finding.0, finding.1.clone());
                    transcript_to_indices.insert(finding.0, vec![finding.1.clone()]);
                }
            }
        }
        transcript_to_indices
    }

    pub fn search(&self, query: impl AsRef<str>, context: usize) -> OrganizedSearchResult {
        let words: Vec<String> = query
            .as_ref()
            .split_whitespace()
            .map(|word| normalize_word(word))
            .filter(|word| word.len() > 0)
            .collect();
        let transcript_indices = self.word_indices_group_by_transcript(&words);
        let allowed_range = words.len() * 2;

        let mut results: OrganizedSearchResult = Map::default();

        for (transcript_id, list_of_word_indices) in transcript_indices {
            let word_segment_ranges = merge_special(list_of_word_indices, allowed_range);
            let transcript_words = self
                .transcript_words
                .get(&transcript_id)
                .expect("It exists");
            for sr in word_segment_ranges {
                let unique_count = sr.set.unique_count();
                let element_count = sr.elements.len();
                let start = if context > sr.min {
                    0
                } else {
                    sr.min - context
                };
                let end = std::cmp::min(sr.max + context, transcript_words.len() - 1);
                let words = transcript_words[start..=end].to_vec();
                let qr = QueryResult::new(transcript_id, words);
                if let Some(unique_group) = results.get_mut(&unique_count) {
                    if let Some(element_group) = unique_group.get_mut(&element_count) {
                        element_group.push(qr);
                    } else {
                        unique_group.insert(element_count, vec![qr]);
                    }
                } else {
                    let mut element_group = Map::default();
                    element_group.insert(element_count, vec![qr]);
                    _ = results.insert(unique_count, element_group);
                }
            }
        }

        results
    }

    pub fn search2(&self, query: impl AsRef<str>, context: usize, page: usize) -> Vec<QueryResult> {
        let words: Vec<String> = query
            .as_ref()
            .split_whitespace()
            .map(|word| normalize_word(word))
            .filter(|word| word.len() > 0)
            .collect();
        let transcript_indices = self.word_indices_group_by_transcript(&words);
        let allowed_range = words.len() * 2;

        let mut results: Map<usize, Map<usize, Vec<(TranscriptId, WordSegmentRange)>>> =
            Map::default();

        for (transcript_id, list_of_word_indices) in transcript_indices {
            let word_segment_ranges = merge_special(list_of_word_indices, allowed_range);
            for sr in word_segment_ranges {
                let unique_count = sr.set.unique_count();
                let element_count = sr.elements.len();
                let qr = (transcript_id, sr);
                if let Some(unique_group) = results.get_mut(&unique_count) {
                    if let Some(element_group) = unique_group.get_mut(&element_count) {
                        element_group.push(qr);
                    } else {
                        unique_group.insert(element_count, vec![qr]);
                    }
                } else {
                    let mut element_group = Map::default();
                    element_group.insert(element_count, vec![qr]);
                    _ = results.insert(unique_count, element_group);
                }
            }
        }

        let window_size = 50;
        let skip_count = page * window_size;
        let take_count = skip_count + window_size;
        let mut page_results = vec![];
        for (transcript_id, sr) in results
            .into_values()
            .rev()
            .flat_map(|m| m.into_values().rev())
            .flatten()
            .skip(skip_count)
            .take(take_count)
        {
            let start = if context > sr.min {
                0
            } else {
                sr.min - context
            };
            let transcript_words = self
                .transcript_words
                .get(&transcript_id)
                .expect("It exists");
            let end = std::cmp::min(sr.max + context, transcript_words.len() - 1);
            let words = transcript_words[start..=end].to_vec();
            page_results.push(QueryResult::new(transcript_id, words));
        }

        page_results
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub transcript_id: usize,
    pub words: Vec<Word>,
}

impl QueryResult {
    pub fn new(transcript_id: usize, words: Vec<Word>) -> Self {
        Self {
            transcript_id,
            words,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub transcript_id: TranscriptId,
    pub segment_range: WordSegmentRange,
    pub words: Vec<Word>,
}

impl SearchResult {
    pub fn new(transcript_id: usize, segment_range: WordSegmentRange, words: Vec<Word>) -> Self {
        Self {
            transcript_id,
            segment_range,
            words,
        }
    }
}
