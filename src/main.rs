pub mod app_config;
pub mod cli;
pub mod hsk_file;
pub mod input_files;
pub mod merge;
pub mod searcher;
pub mod utils;
pub mod word_id;

use std::sync::Arc;

use app_config::AppConfig;
use cli::parse_cli;
use once_cell::sync::Lazy;

pub static CONFIG: Lazy<Arc<AppConfig>> = Lazy::new(|| {
    Arc::new(AppConfig::load().expect("Failed to load config + Failed to create default config"))
});

fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = dbg!(parse_cli());
    Ok(())
}
