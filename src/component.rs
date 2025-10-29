use std::{
    error::Error,
    io::{Write, stdout},
};

use crossterm::{
    ExecutableCommand, cursor,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};

use crate::{
    action::TextActions, input::Cursor, window::{Position, Window, WindowType}
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Buffer,
}

pub struct Component {
    content: Vec<String>,
    component_type: ComponentType,
    editable: bool,
    pub cursor: Cursor,
    pub window: Window,
}

impl Component {
    pub fn new(content: Vec<String>, component_type: ComponentType) -> Component {
        Component {
            content: content.clone(),
            component_type,
            editable: true,
            cursor: Cursor::new(0, 0, false),
            window: Window::new(
                content,
                None,
                None,
                true,
                true,
                WindowType::Tile,
                Position::Center,
                false,
            )
            .unwrap(),
        }
    }
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let mut render_content = self.content.clone();
        if !self.cursor.hidden {
            Component::render_cursor(render_content, &self.cursor)?;
        }
        Ok(())
    }
    pub fn render_cursor(content: Vec<String>, text_cursor: &Cursor) -> Result<(), Box<dyn Error>> {
        let mut stdout = stdout();

        // Clear screen and move cursor to top-left
        stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        stdout.execute(cursor::MoveTo(0, 0))?;

        for (y, line) in content.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if text_cursor.y as usize == y && text_cursor.x as usize == x {
                    stdout.execute(SetBackgroundColor(Color::White))?;
                    stdout.execute(SetForegroundColor(Color::Black))?;
                    stdout.execute(Print(ch))?;
                    stdout.execute(ResetColor)?;
                } else {
                    stdout.execute(Print(ch))?;
                }
            }

            // Cursor at end of line (after last char)
            if text_cursor.y as usize == y && text_cursor.y as usize == line.len() {
                stdout.execute(SetBackgroundColor(Color::White))?;
                stdout.execute(SetForegroundColor(Color::Black))?;
                stdout.execute(Print(" "))?;
                stdout.execute(ResetColor)?;
            }

            stdout.execute(Print("\n"))?;
        }

        stdout.flush()?;
        Ok(())
    }
}

pub fn handle_write_action(
    buffer: Option<&mut Component>,
    action: &TextActions,
) -> Result<(), Box<dyn Error>> {
    let content = match buffer {
        Some(c) => c,
        None => return Ok(()),
    };
    match action {
        TextActions::NewLine => {}
        TextActions::Delete => {}
        TextActions::Insert(c) => {}
    }
    Ok(())
}
