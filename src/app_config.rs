use config::{Config, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{searcher::normalize_word, utils::Mutated};

/// - https://biblehub.com/greek/2147.htm - heuriskó
/// - also where we get heuristic (i assume)
pub const APP_NAME: &'static str = "heurisko";
pub const APP_DISPLAY_NAME: &'static str = "heuriskó";
pub const APP_EXT: &'static str = "hsk";

const DEFAULT_PAGE_SIZE: usize = 50;
const DEFAULT_CONTEXT_SIZE: usize = 20;

const DEFAULT_REMOVE_STOP_WORDS: bool = true;
const DEFAULT_ALLOW_PAGE_SIZE_OVERWRITE: bool = true;
const DEFAULT_ALLOW_CONTEXT_SIZE_OVERWRITE: bool = true;
const DEFAULT_ALLOW_REMOVE_STOP_WORDS_OVERWRITE: bool = true;
const DEFAULT_WORD_DISTANCE: usize = 2;
const DEFAULT_WORD_DISTANCE_WITH_STOP_WORDS_REMOVED: usize = 5;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    data_dir: PathBuf,
    pub page_size: usize,
    pub context_size: usize,
    // Add other config options here
    pub remove_stop_words: bool,
    stop_words_file: Option<PathBuf>,
    pub allow_page_size_overwrite: bool,
    pub allow_context_size_overwrite: bool,
    pub allow_remove_stop_words_overwrite: bool,
    pub word_distance: usize,
    pub word_distance_with_stop_words_removed: usize,
}

impl AppConfig {
    fn create_new() -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir =
            get_data_dir().ok_or_else(|| String::from("Could not determine data directory"))?;
        // Create data dir if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;
        let stop_words_file = get_stop_words_file_path();
        // .ok_or_else(|| String::from("Could not determine stop words directory"))?;
        Ok(Self {
            data_dir,
            page_size: DEFAULT_PAGE_SIZE,
            context_size: DEFAULT_CONTEXT_SIZE,
            remove_stop_words: DEFAULT_REMOVE_STOP_WORDS,
            stop_words_file,
            allow_page_size_overwrite: DEFAULT_ALLOW_PAGE_SIZE_OVERWRITE,
            allow_context_size_overwrite: DEFAULT_ALLOW_CONTEXT_SIZE_OVERWRITE,
            allow_remove_stop_words_overwrite: DEFAULT_ALLOW_REMOVE_STOP_WORDS_OVERWRITE,
            word_distance: DEFAULT_WORD_DISTANCE,
            word_distance_with_stop_words_removed: DEFAULT_WORD_DISTANCE_WITH_STOP_WORDS_REMOVED,
        })
    }

    pub fn load() -> Result<AppConfig, Box<dyn std::error::Error>> {
        load_config()
    }

    pub fn stop_words(&self) -> Option<Vec<String>> {
        let path = self.stop_words_file.as_ref()?.as_path();
        let contents = std::fs::read_to_string(path).ok()?;
        Some(
            contents
                .split_whitespace()
                .map(|word| normalize_word(word))
                .collect(),
        )
    }

    pub fn page_size(&self) -> usize {
        self.page_size
    }

    pub fn context_size(&self) -> usize {
        self.context_size
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }
}

fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", APP_NAME, APP_NAME)
}

fn get_stop_words_file_path() -> Option<PathBuf> {
    Some(get_config_path()?.mutated(|config| config.push("stop_words.txt")))
}

fn get_config_file_path() -> Option<PathBuf> {
    Some(get_config_path()?.mutated(|config| config.push("config.toml")))
}

fn get_config_path() -> Option<PathBuf> {
    get_project_dirs().map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
}

fn get_data_dir() -> Option<PathBuf> {
    get_project_dirs().map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
}

fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path = get_config_file_path()
        .ok_or_else(|| String::from("Could not determine config directory"))?;

    // Create parent directories if they don't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = if config_path.exists() {
        Config::builder()
            .add_source(File::from(config_path))
            .build()?
            .try_deserialize()?
    } else {
        // Create default config if it doesn't exist
        let new_config = AppConfig::create_new()?;
        // Write new config to file
        let toml = toml::to_string_pretty(&new_config)?;
        std::fs::write(&config_path, toml)?;
        // Try adding stop words
        if let Some(stop_words_path) = get_stop_words_file_path() {
            _ = std::fs::write(
                &stop_words_path,
                include_str!("/home/dgmastertemple/.config/heurisko/stop_words.txt"),
            );
        }

        new_config
    };

    Ok(config)
}
