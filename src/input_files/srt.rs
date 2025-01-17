use std::{fmt::Display, path::Path};

use cached::proc_macro::cached;
use regex::Regex;

use crate::hsk_file::{HskResult, Word};

use super::TranscriptFile;

#[cached(size = 1)]
fn srt_regex() -> Regex {
    Regex::new(r"(\d+)\n(\d+):(\d+):(\d+),(\d+) ?--> ?(\d+):(\d+):(\d+),(\d+)\n(.*)\n").unwrap()
}

#[derive(Debug)]
pub struct SrtTime {
    pub hours: u32,
    pub minutes: u32,
    pub seconds: u32,
    pub millis: u32,
}

impl SrtTime {
    pub fn in_seconds(&self) -> f64 {
        let seconds = (self.hours * 60 * 60 + self.minutes * 60 + self.seconds) as f64;
        let millis = self.millis as f64 / 1000.0;
        seconds + millis
    }
}

impl Display for SrtTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02}:{:02}:{:02},{:03}",
            self.hours, self.minutes, self.seconds, self.millis
        )
    }
}

#[derive(Debug)]
pub struct SrtSegment {
    pub id: u32,
    pub start: SrtTime,
    pub end: SrtTime,
    pub text: String,
}

pub struct SrtFile {
    pub segments: Vec<SrtSegment>,
}

impl TranscriptFile for SrtFile {
    fn read(path: &Path) -> HskResult<Self> {
        let contents = std::fs::read_to_string(path)?;
        let mut segments = vec![];
        for cap in srt_regex().captures_iter(&contents) {
            segments.push(SrtSegment {
                id: cap.get(1).unwrap().as_str().parse()?,
                start: SrtTime {
                    hours: cap.get(2).unwrap().as_str().parse()?,
                    minutes: cap.get(3).unwrap().as_str().parse()?,
                    seconds: cap.get(4).unwrap().as_str().parse()?,
                    millis: cap.get(5).unwrap().as_str().parse()?,
                },
                end: SrtTime {
                    hours: cap.get(6).unwrap().as_str().parse()?,
                    minutes: cap.get(7).unwrap().as_str().parse()?,
                    seconds: cap.get(8).unwrap().as_str().parse()?,
                    millis: cap.get(9).unwrap().as_str().parse()?,
                },
                text: cap.get(10).unwrap().as_str().to_string(),
            });
        }
        if segments.len() != 0 {
            Ok(Self { segments })
        } else {
            Err(String::from("`.srt` file must contain at least 1 segment").into())
        }
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
