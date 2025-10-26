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

use crate::{
    input::read_input,
    modal::{Mode, handle_mode_input},
};

mod input;
mod modal;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    stdout.flush()?;

    let mut mode = Mode::Normal;

    println!("Barcode editor ready. Press 'q' to quit.");

    loop {
        handle_mode_input(&mut mode, read_input()?);
        if mode == Mode::Quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show
    )?;

    Ok(())
}
