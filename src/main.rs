use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{
    error::Error,
    io::{Write, stdout},
};

use barcode::{config::generate_default_config, parse_args, run};

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        stdout.flush()?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = stdout();
        let _ = execute!(
            stdout,
            LeaveAlternateScreen,
            DisableMouseCapture,
            cursor::Show
        );
        let _ = stdout.flush();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    if let Some(init_dir) = args.init {
        generate_default_config(init_dir)?;
        return Ok(());
    }

    let _guard = TerminalGuard::new()?;

    run(args)
}
