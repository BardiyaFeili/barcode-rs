use crate::window::{Position, Window, WindowType};

pub struct Buffer {
    content: Vec<String>,
    pub window: Window,
}

impl Buffer {
    pub fn new(content: Vec<String>) -> Buffer {
        Buffer {
            content: content.clone(),
            window: Window::new(
                content,
                None,
                None,
                true,
                true,
                WindowType::Tile,
                Position::Center,
            )
            .unwrap(),
        }
    }
    fn update_window(mut self) {
        self.window.content = self.content;
    }
}
