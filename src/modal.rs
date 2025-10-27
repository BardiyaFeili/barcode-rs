use crate::{component::{Action, TextActions}, input::InputEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
    Quit
}

/// The “mother” function — routes input to the right mode handler.
pub fn handle_mode_input(mode: &mut Mode, event: InputEvent) -> Action {
    match mode {
        Mode::Normal => handle_normal_mode(mode, event),
        Mode::Insert => handle_insert_mode(mode, event),
        Mode::Visual => handle_visual_mode(mode, event),
        Mode::Command => handle_command_mode(mode, event),
        _ => Action::None
    }
}

fn handle_normal_mode(mode: &mut Mode, event: InputEvent) -> Action {
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
        };
    }
    Action::None
}

fn handle_insert_mode(mode: &mut Mode, event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                *mode = Mode::Normal;
                Action::None
            }
            Char(c) => {
                Action::TextAction(TextActions::Insert(c))
            }
            Enter => {
                Action::TextAction(TextActions::NewLine)
            }
            Backspace => {
                Action::TextAction(TextActions::Delete)
            }
            _ => Action::None
        }
    } else {
        Action::None
    }
}

fn handle_visual_mode(mode: &mut Mode, event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                *mode = Mode::Normal;
                println!("Back to NORMAL mode");
            }
            _ => println!("Visual mode input: {:?}", key_event.code),
        };
    }
    Action::None
}

fn handle_command_mode(mode: &mut Mode, event: InputEvent) -> Action {
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
    Action::None
}
