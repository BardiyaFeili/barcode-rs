use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{
    error::Error,
    io::{Write, stdout},
    vec,
};

use crate::{
    buffer::Buffer,
    input::read_input,
    modal::{Mode, handle_mode_input},
    render::draw,
    window::Window,
};

mod buffer;
mod input;
mod modal;
mod render;
mod window;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    stdout.flush()?;

    let mut mode = Mode::Normal;
    let mut active_windows: Vec<Window> = Vec::new();

    println!("Barcode editor ready. Press 'q' to quit.");

    loop {
        handle_mode_input(&mut mode, read_input()?);
        if mode == Mode::Quit {
            break;
        }
        render::render(&mut active_windows)?;
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
