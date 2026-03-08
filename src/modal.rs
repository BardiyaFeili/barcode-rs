use crate::{
    action::{Action, CursorActions, TextActions, WindowActions},
    input::InputEvent,
};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

static INPUT_BUFFER: Mutex<String> = Mutex::new(String::new());

/// The “mother” function — routes input to the right mode handler.
pub fn handle_mode_input(mode: &mut Mode, event: InputEvent) -> Action {
    let action = match mode {
        Mode::Normal => handle_normal_mode(event),
        Mode::Insert => handle_insert_mode(event),
        Mode::Visual => handle_visual_mode(event),
        Mode::Command => handle_command_mode(event),
    };

    match action {
        Action::Mode(new_mode) => *mode = new_mode,
        Action::ExecuteCommand(_) => *mode = Mode::Normal,
        _ => {}
    }

    action
}

fn handle_normal_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::{KeyCode::*, KeyModifiers};

        // Handle Ctrl-w combinations
        if key_event.modifiers.contains(KeyModifiers::CONTROL) && key_event.code == Char('w') {
            return Action::Window(WindowActions::Next);
        }

        match key_event.code {
            Char('i') => Action::Mode(Mode::Insert),
            Char(':') => {
                if let Ok(mut buffer) = INPUT_BUFFER.lock() {
                    buffer.clear();
                }
                Action::Mode(Mode::Command)
            }
            Char('v') => Action::Mode(Mode::Visual),
            Char('q') => Action::Quit,
            Char('k') | Up => Action::Cursor(CursorActions::MoveRel(0, -1)),
            Char('j') | Down => Action::Cursor(CursorActions::MoveRel(0, 1)),
            Char('l') | Right => Action::Cursor(CursorActions::MoveRel(1, 0)),
            Char('h') | Left => Action::Cursor(CursorActions::MoveRel(-1, 0)),
            _ => Action::None,
        }
    } else {
        Action::None
    }
}

fn handle_insert_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => Action::Mode(Mode::Normal),
            Char(c) => Action::Text(TextActions::Insert(c)),
            Enter => Action::Text(TextActions::NewLine),
            Backspace => Action::Text(TextActions::Delete),
            Left => Action::Cursor(CursorActions::MoveRel(-1, 0)),
            Right => Action::Cursor(CursorActions::MoveRel(1, 0)),
            Up => Action::Cursor(CursorActions::MoveRel(0, -1)),
            Down => Action::Cursor(CursorActions::MoveRel(0, 1)),
            _ => Action::None,
        }
    } else {
        Action::None
    }
}

fn handle_visual_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        if key_event.code == Esc {
            return Action::Mode(Mode::Normal);
        }
    }
    Action::None
}

fn handle_command_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => Action::Mode(Mode::Normal),
            Enter => {
                let cmd = if let Ok(mut buffer) = INPUT_BUFFER.lock() {
                    let cmd = buffer.clone();
                    buffer.clear();
                    cmd
                } else {
                    String::new()
                };
                Action::ExecuteCommand(cmd)
            }
            Backspace => {
                if let Ok(mut buffer) = INPUT_BUFFER.lock() {
                    buffer.pop();
                    Action::Command(buffer.clone())
                } else {
                    Action::None
                }
            }
            Char(c) => {
                if let Ok(mut buffer) = INPUT_BUFFER.lock() {
                    buffer.push(c);
                    Action::Command(buffer.clone())
                } else {
                    Action::None
                }
            }
            Left => Action::Cursor(CursorActions::MoveRel(-1, 0)),
            Right => Action::Cursor(CursorActions::MoveRel(1, 0)),
            _ => Action::None,
        }
    } else {
        Action::None
    }
}

#[allow(dead_code)]
pub fn get_input_buffer() -> String {
    if let Ok(buffer) = INPUT_BUFFER.lock() {
        buffer.clone()
    } else {
        String::new()
    }
}
