use crate::input::InputEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Quit
}

/// The “mother” function — routes input to the right mode handler.
pub fn handle_mode_input(mode: &mut Mode, event: InputEvent) {
    match mode {
        Mode::Normal => handle_normal_mode(mode, event),
        Mode::Insert => handle_insert_mode(mode, event),
        Mode::Visual => handle_visual_mode(mode, event),
        Mode::Command => handle_command_mode(mode, event),
        _ => ()
    }
}

fn handle_normal_mode(mode: &mut Mode, event: InputEvent) {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Char('i') => {
                *mode = Mode::Insert;
                println!("Switched to INSERT mode");
            }
            Char(':') => {
                *mode = Mode::Command;
                println!("Switched to COMMAND mode");
            }
            Char('v') => {
                *mode = Mode::Visual;
                println!("Switched to VISUAL mode");
            }
            Char('q') => {
                *mode = Mode::Quit;
            }
            _ => {}
        }
    }
}

fn handle_insert_mode(mode: &mut Mode, event: InputEvent) {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                *mode = Mode::Normal;
                println!("Back to NORMAL mode");
            }
            Char(c) => {
                println!("Insert: typed '{}'", c);
            }
            _ => {}
        }
    }
}

fn handle_visual_mode(mode: &mut Mode, event: InputEvent) {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                *mode = Mode::Normal;
                println!("Back to NORMAL mode");
            }
            _ => println!("Visual mode input: {:?}", key_event.code),
        }
    }
}

fn handle_command_mode(mode: &mut Mode, event: InputEvent) {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                *mode = Mode::Normal;
                println!("Back to NORMAL mode");
            }
            _ => println!("Command mode input: {:?}", key_event.code),
        }
    }
}
