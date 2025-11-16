use crossterm::event::{self, Event, KeyEvent};
use std::error::Error;
use std::io;
use std::time::Duration;

use crate::{action::CursorActions, component::Component, log::log};

pub enum InputEvent {
    Key(KeyEvent),
    None,
}

pub fn read_input() -> io::Result<InputEvent> {
    if event::poll(Duration::from_millis(100))? {
        match event::read()? {
            Event::Key(key) => Ok(InputEvent::Key(key)),
            _ => Ok(InputEvent::None),
        }
    } else {
        Ok(InputEvent::None)
    }
}

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub hidden: bool,
}

impl Cursor {
    pub fn new(x: u16, y: u16, hidden: bool) -> Cursor {
        Cursor { x, y, hidden }
    }
    pub fn move_abs(&mut self, x: Option<u16>, y: Option<u16>, content: &Vec<String>) -> Result<(), Box<dyn Error>> {
        if let Some(y) = y {
            self.y = y;
        }
        if let Some(x) = x {
            if content[self.y as usize].len() <= x as usize {
                self.x = content[self.y as usize].len() as u16 - 1;
            } else {
            self.x = x;
            }
        }
        Ok(())
    }

    pub fn move_rel(&mut self, x: Option<i16>, y: Option<i16>, content: &Vec<String>) -> Result<(), Box<dyn Error>> {
        let new_x = x.map(|dx| {
            let v = self.x as i16 + dx;
            v.max(0) as u16
        });

        let new_y = y.map(|dy| {
            let v = self.y as i16 + dy;
            v.max(0) as u16
        });

        self.move_abs(new_x, new_y, content)
    }
}

pub fn handle_cursor_action(
    component: Option<&mut Component>,
    action: &CursorActions,
) -> Result<(), Box<dyn Error>> {
    log("run handle_cursor_function")?;
    let component = match component {
        Some(c) => c,
        None => return Ok(()),
    };
    match action {
        CursorActions::MoveRel(x, y) => component.cursor.move_rel(Some(x.clone() as i16), Some(y.clone() as i16), &component.content)?,
        _ => {}
    }

    Ok(())
}
