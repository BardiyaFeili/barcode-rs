use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Barcode")]
#[command(
    about = "A minimal TUI text editor written in Rust",
    long_about = None
)]
pub struct Args {
    /// Files to open on startup
    pub files: Vec<String>,
    
    /// Path to the main config.toml override
    #[arg(long, value_name = "FILE")]
    pub config_file: Option<PathBuf>,

    /// Path to the keymap.toml override
    #[arg(long, value_name = "FILE")]
    pub keymap_config: Option<PathBuf>,

    /// Path to the theme.toml override
    #[arg(long, value_name = "FILE")]
    pub theme_config: Option<PathBuf>,

    /// Override the configuration home directory
    #[arg(long, value_name = "DIR")]
    pub config_home: Option<PathBuf>,

    /// Skip requiring global config files to exist
    #[arg(long)]
    pub dont_require_global_configs: bool,
    
    /// Only run startup routines, then exit
    #[arg(long)]
    pub only_startup: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
