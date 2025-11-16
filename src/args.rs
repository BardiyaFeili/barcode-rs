use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(name = "Barcode")]
#[command(about = "A minimal TUI text editor written in Rust", long_about = None)]
pub struct Args {
    pub files: Vec<String>,
    
    #[arg(long)]
    pub config_file: Option<PathBuf>,

    #[arg(long)]
    pub keymap_config: Option<PathBuf>,

    #[arg(long)]
    pub theme_config: Option<PathBuf>,

    #[arg(long)]
    pub config_home: Option<PathBuf>,

    #[arg(long)]
    pub dont_require_global_configs: bool,
    
    #[arg(long)]
    pub only_startup: bool,
}

pub fn parse_args() -> Args {
    Args::parse()
}
