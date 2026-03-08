use crossterm::event::{self, Event, KeyEvent};
use std::error::Error;
use std::io;

use crate::{action::CursorActions, component::Component, modal::Mode, config::Config};

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
    pub target_x: u16,
    #[allow(dead_code)]
    pub hidden: bool,
}

impl Cursor {
    pub fn new(x: u16, y: u16, hidden: bool) -> Cursor {
        Cursor { x, y, target_x: x, hidden }
    }
    pub fn move_abs(
        &mut self,
        x: Option<u16>,
        y: Option<u16>,
        content: &[String],
        mode: &Mode,
        target_v_x: Option<u16>,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(y) = y {
            let max_y = (content.len().saturating_sub(1)) as u16;
            self.y = y.min(max_y);
        }
        if let Some(x) = x {
            let line_len = content[self.y as usize].len() as u16;
            let max_x = match mode {
                Mode::Insert => line_len,
                _ => line_len.saturating_sub(1),
            };

            self.x = x.min(max_x);
            self.target_x = self.x;
        } else if y.is_some() {
            // If moved vertically, try to restore target_x
            let line_len = content[self.y as usize].len() as u16;
            let max_x = match mode {
                Mode::Insert => line_len,
                _ => line_len.saturating_sub(1),
            };
            self.x = self.target_x.min(max_x);
        }

        if let Some(tvx) = target_v_x {
            self.target_x = tvx;
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

        self.move_abs(new_x, new_y, content, mode, None)
    }
}

pub fn handle_cursor_action(
    component: Option<&mut Component>,
    action: &CursorActions,
    mode: &Mode,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let component = match component {
        Some(c) => c,
        None => return Ok(()),
    };

    if let CursorActions::MoveRel(dx, dy) = action {
        component.move_cursor_visual(*dx, *dy, mode, config)?;
    }

    Ok(())
}
