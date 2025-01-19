pub mod app_config;
pub mod cli;
pub mod convert;
pub mod host;
pub mod hsk_file;
pub mod input_files;
pub mod merge;
pub mod searcher;
pub mod utils;
pub mod word_id;

use std::sync::Arc;

use crate::searcher::Searcher;
use app_config::AppConfig;
use clap::{Args, Parser, Subcommand};
use cli::command_cli;
use convert::command_convert;
use host::command_host;
use once_cell::sync::Lazy;

pub static CONFIG: Lazy<Arc<AppConfig>> = Lazy::new(|| {
    Arc::new(AppConfig::load().expect("Failed to load config + Failed to create default config"))
});

pub static SEARCHER: Lazy<Arc<Searcher>> = Lazy::new(|| Arc::new(Searcher::load()));

#[derive(Debug, Parser)]
#[command(author = "Blake Scampone", version = "1.0", about = "heuriskó")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Convert files or directories
#[derive(Debug, Args)]
pub struct CommandConvert {
    /// Path of files or directories to convert
    source: String,
    /// The sub-directory to place the converted files in
    #[arg(short, long)]
    destination: Option<String>,
    /// Whether or not to preserve source folder directory hierarchies
    #[arg(short, long)]
    flatten: bool,
}

#[derive(Debug, Args)]
pub struct CommandHost {
    #[arg(short, long)]
    port: Option<u16>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Convert files or directories
    Convert(CommandConvert),

    /// Run the interactive CLI
    Cli,

    /// Host the web server
    Host(CommandHost),
    // Find,

    // Move,
}

pub fn parse_cli() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert(CommandConvert {
            source,
            flatten,
            destination,
        }) => command_convert(source, destination, flatten)?,

        Commands::Cli => command_cli(),

        Commands::Host(CommandHost { port }) => command_host(port.unwrap_or(8000))?,
    };

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = dbg!(parse_cli());
    Ok(())
}
