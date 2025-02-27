use std::collections::BTreeMap;

use rocket::{
    form::validate::{Contains, Len},
    futures::{stream::iter, StreamExt},
};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::{
    hsk_file::{HskFile, Word},
    merge::{merge_special, WordSegmentRange},
    utils::find_all_extended_words,
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

pub struct Searcher {
    pub transcript_paths: Vec<String>,
    // transcript id -> Word
    pub transcript_words: Map<TranscriptId, Vec<Word>>,
    // word to transcript id
    pub map: WordToTranscriptAndWordIndicesMap,
    pub all_words: Vec<String>,
    pub stop_words: Vec<String>,
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
        let stop_words = match CONFIG.stop_words() {
            Some(stop_words) => stop_words,
            None => vec![],
        };
        let all_words: Vec<String> = map.keys().map(|key| key.to_string()).collect();
        Self {
            transcript_paths,
            transcript_words,
            map,
            all_words,
            stop_words,
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

    pub fn search_exact(&self, query: impl AsRef<str>, page: usize) -> Option<Vec<QueryResult>> {
        let words: Vec<_> = query
            .as_ref()
            .split_whitespace()
            .map(|word| normalize_word(word))
            .filter(|word| word.len() > 0)
            .collect();
        let mut iter = words.iter();

        let mut idx = 1;
        let transcript_starts = {
            // let findings = words.map(|word| self.map.get(&word));
            // what i need is a list of valid transcripts, which starts out as all
            // transcripts that contain the first word
            // but then shrinks as the next one doesn't contain it
            let first = self.map.get(iter.next()?)?;
            let mut valid_transcripts = first
                .iter()
                .map(|(transcript_id, _)| *transcript_id)
                .collect::<Vec<_>>();
            // transcript to word starts
            let mut valid_starts = first
                .into_iter()
                .map(|(tid, word_indices)| (*tid, word_indices.clone()))
                .collect::<BTreeMap<_, _>>();

            while let Some(word) = iter.next() {
                // remove next that aren't part of existing
                let next = self.map.get(word)?;
                let next_transcripts: Vec<_> = next
                    .iter()
                    .filter(|(tid, _)| valid_transcripts.contains(tid))
                    .map(|(tid, _)| *tid)
                    .collect();
                // remove existing valid that aren't continued
                valid_transcripts = valid_transcripts
                    .into_iter()
                    .filter(|vt| next_transcripts.contains(vt))
                    .collect();

                // remove valid starts where transcript isn't continued
                valid_starts = valid_starts
                    .into_iter()
                    .filter(|(key, _)| valid_transcripts.contains(key))
                    // .map(|(key, word_indices)| (key, word_indices.into_iter().filter(|wi| next)))
                    .collect();
                for it in next {
                    let Some(word_indices) = valid_starts.get_mut(&it.0) else {
                        continue;
                    };
                    *word_indices = word_indices
                        .iter()
                        .cloned()
                        .filter(|wi| it.1.contains(*wi + idx))
                        .collect();
                }
                idx += 1;
            }
            valid_starts
        };
        // dbg!(&transcript_starts);
        let mut results = vec![];
        for (transcript, value) in transcript_starts.iter() {
            for start in value.clone() {
                let transcript_words = self.transcript_words.get(transcript).expect("It exists");
                // dbg!(&transcript_words, start, idx);
                let words = transcript_words[start..start + idx]
                    .as_ref()
                    .iter()
                    .map(|word| QueryWord {
                        word: word.word.clone(),
                        start: word.start,
                        end: word.end,
                        matched: true,
                    })
                    .collect();
                let unique_count = idx;
                let element_count = idx;
                let transcript = self
                    .transcript_paths
                    .get(*transcript)
                    .expect("It exists")
                    .clone();

                results.push(QueryResult::new(
                    transcript,
                    words,
                    unique_count,
                    element_count,
                ));
            }
        }
        Some(results)

        // all results are equal
        // let mut results: Vec<(TranscriptId, WordSegmentRange)> = Vec::new();

        // for (transcript_id, list_of_word_indices) in transcript_indices {}
        // todo!()
    }

    pub fn search(
        &self,
        query: impl AsRef<str>,
        context: usize,
        page: usize,
        remove_stop_words: bool,
    ) -> Vec<QueryResult> {
        // perhaps split at more than white space, for example `:` in 1 John 3:10
        let words = query
            .as_ref()
            .split_whitespace()
            .map(|word| normalize_word(word))
            .filter(|word| word.len() > 0);

        let words: Vec<String> = if remove_stop_words {
            words
                .filter(|word| !self.stop_words.contains(word))
                .collect()
        } else {
            words.collect()
        };

        // let words: Vec<String> = words.collect();
        let transcript_indices = self.word_indices_group_by_transcript(&words);
        let allowed_range = if remove_stop_words {
            words.len() * CONFIG.word_distance_with_stop_words_removed
        } else {
            words.len() * CONFIG.word_distance
        };

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

        let page_size = CONFIG.page_size();
        let skip_count = page * page_size;
        let take_count = skip_count + page_size;
        let mut page_results = vec![];
        for (transcript_id, sr) in results
            .into_values()
            // Higher key means higher count of unique elements
            .rev()
            // Higher key means higher count of total elements
            .flat_map(|m| m.into_values().rev())
            .flatten()
            // Take only for current page
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
            // prev
            // let words = transcript_words[start..=end].to_vec();
            // let transcript = self.transcript_paths.get(transcript_id).expect("It exists");
            // page_results.push(QueryResult::new(transcript.clone(), words));
            // new
            let words = transcript_words[start..=end]
                .as_ref()
                .iter()
                .enumerate()
                .map(|(idx, word)| {
                    let this_word_id = idx + start;
                    // let matched = sr.min <= this_word_id && this_word_id <= sr.max;
                    let matched = sr.elements.binary_search(&this_word_id).is_ok();
                    QueryWord {
                        word: word.word.clone(),
                        start: word.start,
                        end: word.end,
                        matched,
                    }
                })
                .collect();
            let unique_count = sr.set.unique_count();
            let element_count = sr.elements.len();
            let transcript = self.transcript_paths.get(transcript_id).expect("It exists");
            page_results.push(QueryResult::new(
                transcript.clone(),
                words,
                unique_count,
                element_count,
            ));
        }

        page_results
    }

    pub fn diagnose_query<'a>(&'a self, query: impl AsRef<str>) -> QueryDiagnostics {
        let words: Vec<String> = query
            .as_ref()
            .split_whitespace()
            .map(|word| normalize_word(word))
            .filter(|word| word.len() > 0)
            .collect();

        let (ignored_words, kept_words): (Vec<String>, Vec<String>) = words
            .clone()
            .into_iter()
            .partition(|word| self.stop_words.contains(word));

        let similar_words: BTreeMap<String, Vec<String>> = words
            .iter()
            .map(|word| {
                (
                    word.clone(),
                    find_all_extended_words(&self.all_words, word).unwrap_or(vec![]),
                )
            })
            .collect();

        let (kept_words, unmatched_words): (Vec<String>, Vec<String>) = kept_words
            .into_iter()
            .partition(|word| self.all_words.binary_search(word).is_ok());

        QueryDiagnostics {
            words,
            ignored_words,
            kept_words,
            unmatched_words,
            similar_words,
        }
    }
    pub fn get_transcript_words(&self, transcript_path: String) -> Option<Vec<Word>> {
        let transcript_id = self
            .transcript_paths
            .iter()
            .position(|path| *path == transcript_path)?;
        let words = self.transcript_words.get(&transcript_id)?;
        Some(words.clone())
    }
}

pub struct QueryParams {
    pub page: usize,
    pub context: usize,
    pub remove_stop_words: bool,
    pub word_distance: usize,
    pub keep_words: Vec<String>,
    /// This means that I should merge these array indices together and treat them as the same word
    /// in the search
    /// - This is basically a selected subset (by the user) of [`QueryDiagnostics::similar_words`]
    pub similar_words: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct QueryDiagnostics {
    pub words: Vec<String>,
    pub ignored_words: Vec<String>,
    pub kept_words: Vec<String>,
    pub unmatched_words: Vec<String>,
    pub similar_words: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryWord {
    pub word: String,
    pub start: Option<f64>,
    pub end: Option<f64>,
    pub matched: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub transcript: String,
    pub words: Vec<QueryWord>,
    pub unique_count: usize,
    pub element_count: usize,
}

impl QueryResult {
    pub fn new(
        transcript: String,
        words: Vec<QueryWord>,
        unique_count: usize,
        element_count: usize,
    ) -> Self {
        Self {
            transcript,
            words,
            unique_count,
            element_count,
        }
    }
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct QueryResult {
//     pub transcript_id: usize,
//     pub words: Vec<Word>,
// }
//
// impl QueryResult {
//     pub fn new(transcript_id: usize, words: Vec<Word>) -> Self {
//         Self {
//             transcript_id,
//             words,
//         }
//     }
// }

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
