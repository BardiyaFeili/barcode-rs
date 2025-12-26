use std::error::Error;

use crossterm::{cursor, style::Stylize};

use crate::{
    action::TextActions,
    component,
    input::Cursor,
    window::{self, Position, Window, WindowType},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Buffer,
}

pub struct Component {
    pub content: Vec<String>,
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
                0,
            )
            .unwrap(),
        }
    }
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let mut render_content = self.content.clone();
        render_content = Component::ready_content(
            render_content,
            self.window.viewpoint,
            self.window.window_height,
        );
        if !self.cursor.hidden {
            self.window.content = Component::render_cursor(render_content, &self.cursor);
        }
        Ok(())
    }
    fn ready_content(content: Vec<String>, viewpoint: usize, window_height: u16) -> Vec<String> {
        let mut built_content: Vec<String> = Vec::new();

        let render_height = (content.len() - viewpoint).min(window_height as usize);

        let built_content: Vec<String> = content
            .iter()
            .skip(viewpoint)
            .take(render_height)
            .cloned()
            .collect();

        built_content
    }
    pub fn render_cursor(content: Vec<String>, text_cursor: &Cursor) -> Vec<String> {
        let mut built_content: Vec<String> = Vec::new();

        for (y, line) in content.iter().enumerate() {
            if y == text_cursor.y as usize {
                let mut built_line: String = String::new();
                for (x, char) in line.chars().enumerate() {
                    if x == text_cursor.x as usize {
                        let colored_char = char.to_string().on_white().black().to_string();
                        built_line.push_str(colored_char.as_str());
                    } else {
                        built_line.push(char)
                    }
                }
                built_content.push(built_line.clone());
            } else {
                built_content.push(line.clone());
            }
        }

        built_content
    }
    fn render_number_line(content: Vec<String>, text_curosr: &Cursor, viewpoint: usize) -> Vec<String> {
        let mut built_content: Vec<String> = Vec::new();

        for (line_number, line) in content.iter().enumerate() {}

        built_content
    }
}

pub fn handle_write_action(
    buffer: Option<&mut Component>,
    action: &TextActions,
) -> Result<(), Box<dyn Error>> {
    let component = match buffer {
        Some(c) => c,
        None => return Ok(()),
    };
    match action {
        TextActions::NewLine => {}
        TextActions::Delete => {}
        TextActions::Insert(c) => {
            component.content[component.cursor.y as usize]
                .insert(component.cursor.x as usize, c.clone());
            component
                .cursor
                .move_rel(Some(1), None, &component.content)?;
        }
    }
    Ok(())
}
