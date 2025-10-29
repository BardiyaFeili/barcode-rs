use crate::{action::{Action, CursorActions, TextActions}, input::InputEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Insert,
    Visual,
    Command,
}

/// The “mother” function — routes input to the right mode handler.
pub fn handle_mode_input(mode: &mut Mode, event: InputEvent) -> Action {
    match mode {
        Mode::Normal => handle_normal_mode(event),
        Mode::Insert => handle_insert_mode(event),
        Mode::Visual => handle_visual_mode(event),
        Mode::Command => handle_command_mode(event),
        _ => Action::None
    }
}


fn handle_normal_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Char('i') => Action::Mode(Mode::Insert),
            Char(':') => Action::Mode(Mode::Command),
            Char('v') => Action::Mode(Mode::Visual),
            Char('q') => Action::Quit,
            Char('k') => Action::CursorAction(CursorActions::MoveRel(0, 1)),
            Char('j') => Action::CursorAction(CursorActions::MoveRel(0, -1)),
            Char('l') => Action::CursorAction(CursorActions::MoveRel(1, 0)),
            Char('h') => Action::CursorAction(CursorActions::MoveRel(-1, 0)),
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
            Esc => {
                Action::Mode(Mode::Normal)
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

fn handle_visual_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                Action::Mode(Mode::Normal)
            }
            _ => Action::None
        };
    }
    Action::None
}

fn handle_command_mode(event: InputEvent) -> Action {
    if let InputEvent::Key(key_event) = event {
        use crossterm::event::KeyCode::*;
        match key_event.code {
            Esc => {
                Action::Mode(Mode::Normal)
            }
            _ => Action::None
        }
    } else {
        Action::None
    }
}
