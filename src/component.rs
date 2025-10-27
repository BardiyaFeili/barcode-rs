use std::error::Error;

use crate::window::{Position, Window, WindowType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType{
    Buffer
}

pub struct Component {
    content: Vec<String>,
    component_type: ComponentType,
    pub window: Window,
}

pub enum Action {
    TextAction(TextActions),
    WindowAction,
    None
}

pub enum TextActions {
    NewLine,
    Insert(char),
    Delete,
}

impl Component {
    pub fn new(content: Vec<String>, component_type: ComponentType) -> Component {
        Component {
            content: content.clone(),
            component_type,
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
    pub fn update(&mut self) {
        self.window.content = self.content.clone();
    }
}

pub fn handle_write_action(buffer: Option<&mut Component>, action: TextActions) -> Result<(), Box<dyn Error>> {
    let content = match buffer {
        Some(c) => c,
        None => {return Ok(())}
    };
    match action{
        TextActions::NewLine => {}
        TextActions::Delete => {}
        TextActions::Insert(c) => {}
    }
    Ok(())
}
