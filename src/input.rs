use crossterm::event::{self, Event, KeyEvent};
use std::error::Error;
use std::io;

use crate::{action::CursorActions, component::Component, modal::Mode};

pub enum InputEvent {
    Key(KeyEvent),
    None,
}

pub fn read_input() -> io::Result<InputEvent> {
    match event::read()? {
        Event::Key(key) => Ok(InputEvent::Key(key)),
        _ => Ok(InputEvent::None),
    }
}

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    #[allow(dead_code)]
    pub hidden: bool,
}

impl Cursor {
    pub fn new(x: u16, y: u16, hidden: bool) -> Cursor {
        Cursor { x, y, hidden }
    }
    pub fn move_abs(
        &mut self,
        x: Option<u16>,
        y: Option<u16>,
        content: &[String],
        mode: &Mode,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(y) = y {
            let max_y = (content.len().saturating_sub(1)) as u16;
            if y > max_y {
                self.y = max_y;
            } else {
                self.y = y;
            }
        }
        if let Some(x) = x {
            let line_len = content[self.y as usize].len() as u16;
            let max_x = match mode {
                Mode::Insert => line_len,
                _ => line_len.saturating_sub(1),
            };

            if x > max_x {
                self.x = max_x;
            } else {
                self.x = x;
            }
        }
        Ok(())
    }

    pub fn move_rel(
        &mut self,
        x: Option<i16>,
        y: Option<i16>,
        content: &[String],
        mode: &Mode,
    ) -> Result<(), Box<dyn Error>> {
        let new_x = x.map(|dx| {
            let v = self.x as i16 + dx;
            v.max(0) as u16
        });

        let new_y = y.map(|dy| {
            let v = self.y as i16 + dy;
            v.max(0) as u16
        });

        self.move_abs(new_x, new_y, content, mode)
    }
}

pub fn handle_cursor_action(
    component: Option<&mut Component>,
    action: &CursorActions,
    mode: &Mode,
) -> Result<(), Box<dyn Error>> {
    let component = match component {
        Some(c) => c,
        None => return Ok(()),
    };
    if let CursorActions::MoveRel(x, y) = action {
        component.cursor.move_rel(Some(*x), Some(*y), &component.content, mode)?;
        component.needs_update = true;
    }

    Ok(())
}
