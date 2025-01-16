use serde::{Deserialize, Serialize};

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
pub struct YouTubeTranscript {
    pub text: String,
    pub start: f64,
    pub duration: f64,
}
