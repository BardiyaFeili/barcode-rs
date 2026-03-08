use std::error::Error;
use std::time::Duration;

use crate::{
    action::TextActions,
    input::Cursor,
    modal::Mode,
    window::{HorizontalAnchor, VerticalAnchor, Window, WindowType},
};

pub struct Component {
    pub content: Vec<String>,
    pub file_path: Option<String>,
    pub component_type: ComponentType,
    #[allow(dead_code)]
    editable: bool,
    pub focusable: bool,
    pub cursor: Cursor,
    pub window: Window,
    pub timer: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Buffer,
    Notification,
    Input,
    StatusLine,
}

impl Component {
    pub fn new(content: Vec<String>, component_type: ComponentType, file_path: Option<String>) -> Component {
        let focusable = match component_type {
            ComponentType::Buffer | ComponentType::Input => true,
            ComponentType::Notification | ComponentType::StatusLine => false,
        };

        Component {
            content: content.clone(),
            file_path,
            component_type,
            editable: true,
            focusable,
            cursor: Cursor::new(0, 0, false),
            window: Window::new(
                content,
                None,
                None,
                true,
                true,
                WindowType::Tile,
                HorizontalAnchor::Left,
                VerticalAnchor::Center,
                false,
                0,
            )
            .unwrap(),
            timer: None,
        }
    }
    
    pub fn with_timer(mut self, duration: Duration) -> Self {
        self.timer = Some(duration);
        self
    }

    pub fn update(&mut self, delta: Duration) -> Result<(), Box<dyn Error>> {
        if let Some(timer) = &mut self.timer {
            *timer = timer.saturating_sub(delta);
        }

        let render_content = Component::ready_content(
            &self.content,
            self.window.viewpoint,
            self.window.window_height,
        );

        self.window.content = render_content;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        self.timer.map_or(false, |t| t.is_zero())
    }

    fn ready_content(content: &[String], viewpoint: usize, window_height: u16) -> Vec<String> {
        let render_height = content
            .len()
            .saturating_sub(viewpoint)
            .min(window_height as usize);

        content
            .iter()
            .skip(viewpoint)
            .take(render_height)
            .cloned()
            .collect()
    }
}

pub fn handle_write_action(
    buffer: Option<&mut Component>,
    action: &TextActions,
    mode: &Mode,
) -> Result<(), Box<dyn Error>> {
    let component = match buffer {
        Some(c) => c,
        None => return Ok(()),
    };
    if !component.focusable {
        return Ok(());
    }
    match action {
        TextActions::NewLine => {
            let x = component.cursor.x as usize;
            let y = component.cursor.y as usize;
            let line = &component.content[y];
            let new_line = line[x..].to_string();
            component.content[y] = line[..x].to_string();
            component.content.insert(y + 1, new_line);
            component.cursor.y += 1;
            component.cursor.x = 0;
        }
        TextActions::Delete => {
            let x = component.cursor.x as usize;
            let y = component.cursor.y as usize;

            if x > 0 {
                // Remove character before cursor
                component.content[y].remove(x - 1);
                component.cursor.x -= 1;
            } else if y > 0 {
                // Join with previous line
                let current_line = component.content.remove(y);
                let prev_line = &mut component.content[y - 1];
                let prev_len = prev_line.len() as u16;
                prev_line.push_str(&current_line);
                component.cursor.y -= 1;
                component.cursor.x = prev_len;
            }
        }
        TextActions::Insert(c) => {
            component.content[component.cursor.y as usize].insert(component.cursor.x as usize, *c);
            component
                .cursor
                .move_rel(Some(1), None, &component.content, mode)?;
        }
    }
    Ok(())
}
