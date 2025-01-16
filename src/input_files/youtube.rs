use serde::{Deserialize, Serialize};

use crate::hsk_file::Word;

use super::TranscriptFile;

/**
Source: https://pypi.org/project/youtube-transcript-api/

Installation: `pip install youtube-transcript-api`

Usage:
```py
from youtube_transcript_api import YouTubeTranscriptApi

YouTubeTranscriptApi.get_transcript(video_id)
```
*/
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YouTubeTranscriptFile(Vec<YouTubeTranscriptSegment>);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct YouTubeTranscriptSegment {
    pub text: String,
    pub start: f64,
    pub duration: f64,
}

impl TranscriptFile for YouTubeTranscriptFile {
    fn read(path: &std::path::Path) -> crate::hsk_file::HskResult<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    fn into_words(self) -> crate::hsk_file::HskResult<crate::hsk_file::Words> {
        let mut words = self
            .0
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
        if let Some(last) = self.0.last() {
            words.extend(last.text.split_whitespace().map(|word| Word {
                word: word.to_string(),
                start: Some(last.start),
                end: None,
            }));
        }
        Ok(words)
    }
}
