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

    /// Only run startup routines, then exit
    #[arg(long)]
    pub only_startup: bool,

    /// Generate default configuration files in the specified directory
    #[arg(long, value_name = "DIR", num_args = 0..=1)]
    pub init: Option<Option<PathBuf>>,
}

pub fn parse_args() -> Args {
    Args::parse()
}
