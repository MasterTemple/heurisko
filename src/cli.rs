use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

use crate::app_config::{APP_DISPLAY_NAME, APP_EXT};
use crate::hsk_file::{HskFile, HskResult};
use crate::searcher::Searcher;
use crate::utils::{prompt, Timer};
use crate::CONFIG;

pub fn command_cli() {
    let mut timer = Timer::new();
    let searcher = Searcher::load();
    timer.print(
        format!(
            "Searcher loaded {} transcripts",
            searcher.transcript_words.len(),
        )
        .as_str(),
    );

    loop {
        println!("");
        let input = prompt("Search: ");
        println!("");
        if input.as_str() == "exit" {
            break;
        }
        timer.reset();
        let results = searcher.search(input, CONFIG.context_size(), 0, true);
        timer.print(format!("Query Complete").as_str());
        for result in results {
            let start = result.words.iter().find_map(|w| w.start).unwrap_or(0.0);
            let end = result
                .words
                .iter()
                .find_map(|w| w.end.map(|e| e.to_string()))
                .unwrap_or_default();
            let text = result
                .words
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            println!("[{}: {}..{}] {}", &result.transcript, start, end, text);
        }
    }
}
