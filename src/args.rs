use clap::Parser;

#[derive(Parser)]
#[command(name = "Barcode")]
#[command(about = "A minimal TUI text editor written in Rust", long_about = None)]
pub struct Args {
    pub files: Vec<String>,
}

pub fn parse_args() -> Args {
    Args::parse()
}
