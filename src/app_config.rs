use config::{Config, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// - https://biblehub.com/greek/2147.htm - heuriskó
/// - also where we get heuristic (i assume)
pub const APP_NAME: &'static str = "heurisko";
pub const APP_DISPLAY_NAME: &'static str = "heuriskó";
pub const APP_EXT: &'static str = "hsk";

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    data_dir: PathBuf,
    // Add other config options here
}

impl AppConfig {
    fn create_new() -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir =
            get_data_dir().ok_or_else(|| String::from("Could not determine data directory"))?;
        // Create data dir if it doesn't exist
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self { data_dir })
    }
    pub fn load() -> Result<AppConfig, Box<dyn std::error::Error>> {
        load_config()
    }
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }
}

fn get_project_dirs() -> Option<ProjectDirs> {
    ProjectDirs::from("com", APP_NAME, APP_NAME)
}

fn get_config_path() -> Option<PathBuf> {
    get_project_dirs().map(|proj_dirs| proj_dirs.config_dir().join("config.toml"))
}

fn get_data_dir() -> Option<PathBuf> {
    get_project_dirs().map(|proj_dirs| proj_dirs.data_dir().to_path_buf())
}

fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let config_path =
        get_config_path().ok_or_else(|| String::from("Could not determine config directory"))?;

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
        let toml = toml::to_string_pretty(&new_config)?;
        std::fs::write(&config_path, toml)?;
        new_config
    };

    Ok(config)
}
