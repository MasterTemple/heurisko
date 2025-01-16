use clap::{Parser, Subcommand};
use once_cell::sync::Lazy;
use rocket::response::status::BadRequest;
use rocket::tokio::runtime::Runtime;
use rocket::{get, routes};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

use crate::app_config::{APP_DISPLAY_NAME, APP_EXT};
use crate::hsk_file::{HskFile, HskResult};
use crate::searcher::Searcher;
use crate::utils::{prompt, Timer};
use crate::CONFIG;

#[derive(Debug, Parser)]
#[command(author = "Your Name", version = "1.0", about = "A versatile CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Convert files or directories
    Convert {
        /// Path of files or directories to convert
        source: String,
        /// The sub-directory to place the converted files in
        #[arg(short, long)]
        destination: Option<String>,
        /// Whether or not to preserve source folder directory hierarchies
        #[arg(short, long)]
        flatten: bool,
    },

    /// Run the interactive CLI
    Cli,

    /// Host the web server
    Host,
    // Find,

    // Move,
}

fn command_convert(source: String, destination: Option<String>, flatten: bool) -> HskResult<()> {
    let mut data_dir = CONFIG.data_dir();
    let source = Path::new(&source);
    let dest = destination.unwrap_or(String::new());
    let dest = Path::new(&dest);
    data_dir.push(dest);

    if source.is_file() {
        print!("File: ");
        let mut dest = data_dir.join(source.file_name().unwrap());
        dest.set_extension(APP_EXT);
        println!("Converting: {source:?} -> {dest:?}\n");
        HskFile::convert(source, dest.as_path())?;
    }
    if source.is_dir() {
        println!("Directory:");
        let walker = WalkDir::new(source);
        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let mut dest = data_dir.clone();
                if flatten {
                    dest.push(path.file_name().unwrap());
                } else {
                    dest.push(path.strip_prefix(source).unwrap());
                }
                dest.set_extension(APP_EXT);
                println!("Converting: {path:?} -> {dest:?}\n");
                HskFile::convert(path, dest.as_path())?;
            }
        }
    }
    Ok(())
}

fn command_cli() {
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
        let results = searcher.search(input, 5);
        timer.print(format!("Query Complete").as_str());
        let Some(unique_group_key) = results.keys().max() else {
            continue;
        };
        let unique_group = results.get(&unique_group_key).unwrap();
        let Some(element_group_key) = unique_group.keys().max() else {
            continue;
        };
        let element_group = unique_group.get(element_group_key).unwrap();
        for result in element_group {
            let text = result
                .words
                .iter()
                .map(|w| w.word.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            let start = result.words.iter().find_map(|w| w.start).unwrap_or(0.0);
            let end = result
                .words
                .iter()
                .find_map(|w| w.end.map(|e| e.to_string()))
                .unwrap_or_default();

            println!(
                "[{:?}: {}..{}] {text}",
                &result.transcript_id,
                // searcher
                //     .transcript_paths
                //     .get(&result.transcript_id)
                //     .unwrap()
                //     .file_stem()
                //     .unwrap()
                //     .to_str()
                //     .unwrap(),
                start,
                end
            );
        }
    }
}

pub fn parse_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert {
            source,
            flatten,
            destination,
        } => command_convert(source, destination, flatten)?,

        Commands::Cli => {
            command_cli();
        }

        Commands::Host => {
            // Create a new tokio runtime
            let rt = Runtime::new()?;

            // Launch rocket in the runtime
            _ = rt.block_on(async {
                let rocket = rocket::build().mount("/", routes![index, search, ids]);
                rocket
                    .launch()
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                Ok::<(), Box<dyn std::error::Error>>(())
            })?;
        }
    };

    Ok(())
}

// Example route handlers
#[get("/")]
async fn index() -> &'static str {
    APP_DISPLAY_NAME
}

pub static SEARCHER: Lazy<Arc<Searcher>> = Lazy::new(|| Arc::new(Searcher::load()));

#[get("/ids")]
async fn ids() -> String {
    serde_json::to_string(
        &SEARCHER
            .transcript_paths
            .iter()
            .enumerate()
            .map(|(id, path)| (id, path))
            .collect::<BTreeMap<_, _>>(),
    )
    .expect("This can serialize")
}

#[get("/search?<query>&<context>&<page>")]
async fn search(
    query: String,
    context: Option<usize>,
    page: Option<usize>,
) -> Result<String, BadRequest<String>> {
    let results = SEARCHER.search(query, context.unwrap_or(5));
    let page = page.unwrap_or(0);
    let window_size = 50;
    let skip_count = page * window_size;
    let take_count = skip_count + window_size;
    let mut page_results = vec![];
    for value in results
        .into_values()
        .rev()
        .flat_map(|m| m.into_values().rev())
        .flatten()
        .skip(skip_count)
        .take(take_count)
    {
        page_results.push(value);
    }
    serde_json::to_string(&page_results).map_err(|err| BadRequest(err.to_string()))
}
