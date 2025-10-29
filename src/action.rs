use std::error::Error;

use crate::{component::{Component, handle_write_action}, input::handle_cursor_action, log::log, modal::Mode};

#[derive(Debug, PartialEq)]
pub enum Action {
    TextAction(TextActions),
    CursorAction(CursorActions),
    WindowAction,
    Mode(Mode),
    Quit,
    None,
}

#[derive(Debug, PartialEq)]
pub enum TextActions {
    NewLine,
    Insert(char),
    Delete,
}

#[derive(Debug, PartialEq)]
pub enum CursorActions {
    MoveAbs(u16, u16),
    MoveRel(i16, i16),
}

pub fn take_action(action: &Action, last: usize, active_components: &mut Vec<Component>) -> Result<(), Box<dyn Error>> {
    if action != &Action::None {
        log(format!("{:?}", action).as_str())?;
    }
    match action {
        Action::TextAction(a) => handle_write_action(active_components.get_mut(last - 1), a)?,
        Action::CursorAction(a) => {
            handle_cursor_action(active_components.get_mut(last - 1), a)?
        }
        _ => (),
    }
    Ok(())
}