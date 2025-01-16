use std::{fmt::Display, path::Path};

use cached::proc_macro::cached;
use regex::Regex;

use crate::hsk_file::{HskResult, Word};

use super::TranscriptFile;

#[cached(size = 1)]
fn sbv_regex() -> Regex {
    Regex::new(r"(\d+):(\d+):(\d+).(\d+),(\d+):(\d+):(\d+).(\d+)\n(.*)\n").unwrap()
}

#[derive(Debug)]
pub struct SbvTime {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub millis: u32,
}

impl SbvTime {
    pub fn in_seconds(&self) -> f64 {
        let seconds = (self.hours * 60 * 60 + self.minutes * 60 + self.seconds) as f64;
        let millis = self.millis as f64 / 1000.0;
        seconds + millis
    }
}

impl Display for SbvTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}",
            self.hours, self.minutes, self.seconds, self.millis
        )
    }
}

#[derive(Debug)]
pub struct SbvSegment {
    pub start: SbvTime,
    pub end: SbvTime,
    pub text: String,
}

pub struct SbvFile {
    pub segments: Vec<SbvSegment>,
}

impl TranscriptFile for SbvFile {
    fn read(path: &Path) -> HskResult<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut segments = vec![];
        for cap in sbv_regex().captures_iter(&contents) {
            segments.push(SbvSegment {
                start: SbvTime {
                    hours: cap.get(1).unwrap().as_str().parse()?,
                    minutes: cap.get(2).unwrap().as_str().parse()?,
                    seconds: cap.get(3).unwrap().as_str().parse()?,
                    millis: cap.get(4).unwrap().as_str().parse()?,
                },
                end: SbvTime {
                    hours: cap.get(5).unwrap().as_str().parse()?,
                    minutes: cap.get(6).unwrap().as_str().parse()?,
                    seconds: cap.get(7).unwrap().as_str().parse()?,
                    millis: cap.get(8).unwrap().as_str().parse()?,
                },
                text: cap.get(9).unwrap().as_str().to_string(),
            });
        }
        Ok(Self { segments })
    }

    fn into_words(self) -> HskResult<crate::hsk_file::Words> {
        Ok(self
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
            .collect())
    }
}
