use config::{Config, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{hsk_file::HskResult, searcher::normalize_word, utils::Mutated};

// store my defaults from `config.toml` in the binary
// try to read from the default path, if i can't
// create the default

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

// pub const APP_NAME: &'static str = "heurisko";
// pub const APP_DISPLAY_NAME: &'static str = "heuriskó";
// pub const APP_EXT: &'static str = "hsk";

#[derive(Debug, Serialize, Deserialize)]
pub struct MyConfig {
    pub paths: FilePaths,
    pub parameters: Parameters,
    pub host: HostConfig,
}

impl MyConfig {
    fn get_project_dirs() -> Option<ProjectDirs> {
        ProjectDirs::from("com", APP_NAME, APP_NAME)
    }

    fn get_default_data_path() -> Option<PathBuf> {
        Some(Self::get_project_dirs()?.data_dir().to_path_buf())
    }

    fn get_default_config_path() -> Option<PathBuf> {
        Some(Self::get_project_dirs()?.config_dir().to_path_buf())
    }

    fn get_default_stop_words_path() -> Option<PathBuf> {
        Some(
            Self::get_default_config_path()?
                .mutated(|config| config.push("/parameters/stop_words.txt")),
        )
    }

    fn get_default_word_endings_path() -> Option<PathBuf> {
        Some(
            Self::get_default_config_path()?
                .mutated(|config| config.push("/parameters/word_endings.txt")),
        )
    }

    fn new() -> Self {
        toml::from_str(include_str!("../config/config.toml"))
            .expect("My default config should be valid")
    }

    fn load(config_path_overwrite: Option<String>) -> HskResult<Self> {
        let config_path = match config_path_overwrite {
            Some(path) => PathBuf::from(path),
            None => Self::get_default_config_path()
                .ok_or_else(|| String::from("Could not determine config directory"))?,
        };
        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // try reading config file
        let config = if config_path.exists() {
            Config::builder()
                .add_source(File::from(config_path))
                .build()?
                .try_deserialize()?
        }
        // or create default file
        else {
            let mut new_config = MyConfig::new();
            new_config.paths.data = Self::get_default_data_path()
                .ok_or_else(|| String::from("Could not determine config directory"))?;
            new_config.paths.stop_words = Self::get_default_stop_words_path();
            new_config.paths.word_endings = Self::get_default_word_endings_path();

            let toml = toml::to_string_pretty(&new_config)?;
            std::fs::write(&config_path, toml)?;

            // Try adding stop words
            if let Some(stop_words_path) = &new_config.paths.stop_words {
                _ = stop_words_path
                    .parent()
                    .map(|parent| std::fs::create_dir_all(parent));
                _ = std::fs::write(&stop_words_path, include_str!("../config/stop_words.txt"));
            }

            // Try adding word endings
            if let Some(word_endings_path) = &new_config.paths.word_endings {
                _ = word_endings_path
                    .parent()
                    .map(|parent| std::fs::create_dir_all(parent));
                _ = std::fs::write(
                    &word_endings_path,
                    include_str!("../config/word_endings.txt"),
                );
            }

            new_config
        };
        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FilePaths {
    /// All `.hsk` files will be stored/read from this directory
    pub data: PathBuf,
    /// Words such as "the", "an", and more, that should be removed from the search query when the corresponding parameter (`remove_stop_words.value`) is set to true (`\n` delimited)
    pub stop_words: Option<PathBuf>,
    /// - Word suffixes, such as "tion", "ness" and more, to be removed from the end of input words when providing suggestions for similar words (`\n` delimited)
    /// - These are treated as Regular Expressions
    pub word_endings: Option<PathBuf>,
}

impl Default for FilePaths {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OverwritableParameter<T> {
    /// The parameter's actual value
    pub value: T,
    /// Whether or not the user can, with their API request, overwrite the default value (above)
    pub overwritable: bool,
}
impl<T> OverwritableParameter<T> {
    fn create(value: T) -> Self {
        Self {
            value,
            overwritable: true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameters {
    /// Amount of results to display per paginated result
    pub page_size: OverwritableParameter<usize>,
    /// Amount of words to include on both the left and right side of the matched segment as context for the search result
    pub context_size: OverwritableParameter<usize>,
    /// Whether or not "stop words" (such as "the", "an", ...) should be removed from the search query
    pub remove_stop_words: OverwritableParameter<bool>,
    /// The minimum distance between words for them to be grouped in the same segment
    pub word_distance: OverwritableParameter<usize>,
    /// The minimum distance between words for them to be grouped in the same segment when stop words are removed
    pub word_distance_with_stop_words_removed: OverwritableParameter<usize>,
}

impl Default for Parameters {
    fn default() -> Self {
        Self {
            page_size: OverwritableParameter::create(50),
            context_size: OverwritableParameter::create(20),
            remove_stop_words: OverwritableParameter::create(true),
            word_distance: OverwritableParameter::create(2),
            word_distance_with_stop_words_removed: OverwritableParameter::create(5),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HostConfig {
    /// The port which the server will be hosted at
    pub port: u16,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self { port: 8000 }
    }
}
