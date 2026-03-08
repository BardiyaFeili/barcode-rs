use crate::{
    action::{TextActions, PromptAction},
    config::Config,
    input::Cursor,
    modal::Mode,
    window::{HorizontalAnchor, VerticalAnchor, Window, WindowType},
    highlight::SyntaxHighlighter,
};
use std::error::Error;
use std::time::Duration;
use unicode_width::UnicodeWidthChar;
use crossterm::style::Color;

pub struct Component {
    pub content: Vec<String>,
    pub highlights: Vec<Vec<Option<Color>>>,
    pub file_path: Option<String>,
    pub component_type: ComponentType,
    pub prompt_action: Option<PromptAction>,
    pub modified: bool,
    #[allow(dead_code)]
    editable: bool,
    pub focusable: bool,
    pub cursor: Cursor,
    pub window: Window,
    pub timer: Option<Duration>,
    pub needs_update: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentType {
    Buffer,
    Notification,
    Input,
    StatusLine,
}

impl Component {
    pub fn new(
        content: Vec<String>,
        component_type: ComponentType,
        file_path: Option<String>,
        config: &Config,
    ) -> Component {
        let focusable = match component_type {
            ComponentType::Buffer | ComponentType::Input => true,
            ComponentType::Notification | ComponentType::StatusLine => false,
        };

        let (window_type, h_anchor, v_anchor) = match component_type {
            ComponentType::Buffer => (
                WindowType::Tile,
                HorizontalAnchor::Left,
                VerticalAnchor::Center,
            ),
            ComponentType::Notification => (
                WindowType::Floating,
                HorizontalAnchor::Right,
                VerticalAnchor::Top,
            ),
            ComponentType::Input => (
                WindowType::Floating,
                HorizontalAnchor::Center,
                VerticalAnchor::Bottom,
            ),
            ComponentType::StatusLine => (
                WindowType::Tile,
                HorizontalAnchor::Left,
                VerticalAnchor::Bottom,
            ),
        };

        let mut comp = Component {
            content: content.clone(),
            highlights: vec![Vec::new(); content.len()],
            file_path,
            component_type,
            prompt_action: None,
            modified: false,
            editable: true,
            focusable,
            cursor: Cursor::new(0, 0, false),
            window: Window::new(
                content,
                None,
                None,
                true,
                true,
                window_type,
                h_anchor,
                v_anchor,
                false,
                0,
            )
            .unwrap(),
            timer: None,
            needs_update: true,
        };

        comp.prepare_view(config);
        comp
    }

    pub fn with_timer(mut self, duration: Duration) -> Self {
        self.timer = Some(duration);
        self
    }

    pub fn with_prompt_action(mut self, action: PromptAction) -> Self {
        self.prompt_action = Some(action);
        self
    }

    pub fn update(&mut self, delta: Duration, config: &Config, highlighter: &mut SyntaxHighlighter) -> Result<(), Box<dyn Error>> {
        if let Some(timer) = &mut self.timer {
            *timer = timer.saturating_sub(delta);
        }

        // Only update window content if necessary
        if self.needs_update || self.window.content.is_empty() {
            self.update_highlights(highlighter);
            self.prepare_view(config);
            self.needs_update = false;
        }

        Ok(())
    }

    fn update_highlights(&mut self, highlighter: &mut SyntaxHighlighter) {
        if self.component_type != ComponentType::Buffer {
            return;
        }

        let extension = self.file_path.as_ref()
            .and_then(|p| std::path::Path::new(p).extension())
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let full_content = self.content.join("\n");
        self.highlights = self.content.iter()
            .map(|l| vec![None; l.chars().count()])
            .collect();

        if let Ok(highlights) = highlighter.highlight(&full_content, extension) {
            let mut line_start_byte = 0;
            let mut highlight_idx = 0;

            for (line_idx, line_str) in self.content.iter().enumerate() {
                let line_end_byte = line_start_byte + line_str.len();

                while highlight_idx < highlights.len() && highlights[highlight_idx].1 <= line_start_byte {
                    highlight_idx += 1;
                }

                let mut h_idx = highlight_idx;
                while h_idx < highlights.len() && highlights[h_idx].0 < line_end_byte {
                    let (h_start, h_end, color) = highlights[h_idx];
                    let s = h_start.max(line_start_byte) - line_start_byte;
                    let e = h_end.min(line_end_byte) - line_start_byte;

                    if s < e {
                        let mut char_idx = 0;
                        let mut byte_offset = 0;
                        for c in line_str.chars() {
                            if byte_offset >= s && byte_offset < e {
                                if char_idx < self.highlights[line_idx].len() {
                                    self.highlights[line_idx][char_idx] = Some(color);
                                }
                            }
                            byte_offset += c.len_utf8();
                            char_idx += 1;
                            if byte_offset >= e { break; }
                        }
                    }
                    h_idx += 1;
                }
                line_start_byte = line_end_byte + 1;
            }
        }
    }

    fn prepare_view(&mut self, config: &Config) {
        let window_height = self.window.window_height as usize;
        
        let gutter_width = if self.component_type == ComponentType::Buffer {
            match config.line_number.mode {
                crate::config::LineNumberMode::None => 0,
                crate::config::LineNumberMode::Absolute | crate::config::LineNumberMode::Relative => {
                    let max_lines = self.content.len();
                    let digits = max_lines.to_string().len().max(2);
                    digits + config.line_number.padding_left + config.line_number.padding_right
                }
            }
        } else {
            0
        } as u16;

        self.window.gutter_width = gutter_width;
        let window_width = self.window.window_width.saturating_sub(gutter_width).saturating_sub(1) as usize;
        
        if config.editor.wrap {
            let mut wrapped_lines = Vec::new();
            let mut wrapped_highlights = Vec::new();
            let mut visual_info = Vec::new(); // (physical_y, physical_x_start, visual_y, is_first)
            let mut cursor_wrapped_y = 0;
            let mut cursor_wrapped_x = 0;

            for (y, line) in self.content.iter().enumerate() {
                let mut is_first = true;
                if line.is_empty() {
                    if y == self.cursor.y as usize {
                        cursor_wrapped_y = wrapped_lines.len();
                        cursor_wrapped_x = 0;
                    }
                    visual_info.push((y, 0, wrapped_lines.len(), true));
                    wrapped_lines.push(String::new());
                    wrapped_highlights.push(Vec::new());
                    continue;
                }

                let mut current_wrapped_line = String::new();
                let mut current_wrapped_highlights = Vec::new();
                let mut current_w = 0;
                let mut start_x = 0;

                for (x, c) in line.chars().enumerate() {
                    let cw = c.width().unwrap_or(0);
                    let char_highlight = self.highlights[y].get(x).cloned().flatten();

                    if y == self.cursor.y as usize && x == self.cursor.x as usize {
                        cursor_wrapped_y = wrapped_lines.len();
                        cursor_wrapped_x = current_w;
                    }

                    if current_w + cw > window_width && !current_wrapped_line.is_empty() {
                        visual_info.push((y, start_x, wrapped_lines.len(), is_first));
                        is_first = false;
                        wrapped_lines.push(current_wrapped_line);
                        wrapped_highlights.push(current_wrapped_highlights);
                        current_wrapped_line = String::new();
                        current_wrapped_highlights = Vec::new();
                        current_w = 0;
                        start_x = x;
                        
                        if y == self.cursor.y as usize && x == self.cursor.x as usize {
                            cursor_wrapped_y = wrapped_lines.len();
                            cursor_wrapped_x = 0;
                        }
                    }

                    current_wrapped_line.push(c);
                    current_wrapped_highlights.push(char_highlight);
                    current_w += cw;
                }
                
                if y == self.cursor.y as usize && self.cursor.x as usize == line.len() {
                    cursor_wrapped_y = wrapped_lines.len();
                    cursor_wrapped_x = current_w;
                }

                visual_info.push((y, start_x, wrapped_lines.len(), is_first));
                wrapped_lines.push(current_wrapped_line);
                wrapped_highlights.push(current_wrapped_highlights);
            }

            self.window.visual_cursor_info = Some(visual_info);
            self.fix_viewpoint(window_height, cursor_wrapped_y, config.editor.margin);

            let start = self.window.viewpoint;
            let end = (start + window_height).min(wrapped_lines.len());
            self.window.content = if start < wrapped_lines.len() {
                wrapped_lines[start..end].to_vec()
            } else {
                Vec::new()
            };
            self.window.highlights = if start < wrapped_highlights.len() {
                wrapped_highlights[start..end].to_vec()
            } else {
                Vec::new()
            };

            self.window.visual_cursor = Some((cursor_wrapped_x as u16, cursor_wrapped_y as u16));
        } else {
            // No wrapping logic
            self.window.visual_cursor = None;
            
            let mut visual_info = Vec::new();
            for y in 0..self.content.len() {
                visual_info.push((y, 0, y, true));
            }
            self.window.visual_cursor_info = Some(visual_info);

            let cursor_y = self.cursor.y as usize;
            self.fix_viewpoint(window_height, cursor_y, config.editor.margin);

            let start = self.window.viewpoint;
            let end = (start + window_height).min(self.content.len());
            self.window.content = self.content[start..end].to_vec();
            self.window.highlights = self.highlights[start..end].to_vec();
        }
    }

    fn fix_viewpoint(&mut self, window_height: usize, cursor_y: usize, margin: usize) {
        let effective_margin = margin.min(window_height.saturating_sub(1) / 2);

        // Scroll up (viewpoint decreases)
        // Only scroll up if cursor is above current view + margin
        if cursor_y < self.window.viewpoint + effective_margin {
            while cursor_y < self.window.viewpoint + effective_margin && self.window.viewpoint > 0 {
                self.window.viewpoint -= 1;
            }
        }

        // Scroll down (viewpoint increases)
        // Only scroll down if cursor is below current view + height - margin
        if cursor_y >= self.window.viewpoint + window_height.saturating_sub(effective_margin) {
            while cursor_y >= self.window.viewpoint + window_height.saturating_sub(effective_margin) {
                self.window.viewpoint += 1;
            }
        }
        
        // Final safety check: if cursor is STILL out of bounds after margin logic
        // (can happen if window shrunk significantly)
        if cursor_y < self.window.viewpoint {
            self.window.viewpoint = cursor_y;
        } else if cursor_y >= self.window.viewpoint + window_height {
            self.window.viewpoint = cursor_y.saturating_sub(window_height).saturating_add(1);
        }
    }

    pub fn move_cursor_visual(&mut self, dx: i16, dy: i16, mode: &Mode, config: &Config) -> Result<(), Box<dyn Error>> {
        if config.editor.wrap && self.window.visual_cursor_info.is_some() {
            if let Some(info) = &self.window.visual_cursor_info {
                let (_, v_y) = self.window.visual_cursor.unwrap_or((0, 0));
                
                if dy != 0 {
                    let target_v_y = if dy > 0 {
                        (v_y as usize).saturating_add(dy as usize).min(info.last().map(|i| i.2).unwrap_or(0))
                    } else {
                        (v_y as usize).saturating_sub(dy.unsigned_abs() as usize)
                    };

                    // Find visual info for the target visual line
                    if let Some(&(p_y, p_x_start, _, _)) = info.iter().find(|i| i.2 == target_v_y) {
                        let line = &self.content[p_y];
                        let mut current_vw = 0;
                        let mut byte_idx = p_x_start;
                        
                        let next_v_line_start_x = info.iter()
                            .find(|i| i.2 == target_v_y + 1 && i.0 == p_y)
                            .map(|i| i.1)
                            .unwrap_or(line.len());

                        for c in line[p_x_start..next_v_line_start_x].chars() {
                            use unicode_width::UnicodeWidthStr;
                            let s = c.to_string();
                            let cw = UnicodeWidthStr::width(s.as_str());
                            if current_vw + cw > self.cursor.target_x as usize {
                                break;
                            }
                            current_vw += cw;
                            byte_idx += c.len_utf8();
                        }
                        self.cursor.y = p_y as u16;
                        self.cursor.x = byte_idx as u16;
                    }
                } else if dx != 0 {
                    // Horizontal movement on wrapped lines
                    let current_p_y = self.cursor.y as usize;
                    let current_p_x = self.cursor.x as usize;
                    let line = &self.content[current_p_y];
                    
                    if dx > 0 {
                        // Move Right
                        let max_x = if *mode == Mode::Insert { line.len() } else { line.len().saturating_sub(1) };
                        if current_p_x < max_x {
                            if let Some(c) = line[current_p_x..].chars().next() {
                                self.cursor.x += c.len_utf8() as u16;
                                self.cursor.target_x = self.calculate_visual_x(current_p_y, self.cursor.x as usize, info);
                            }
                        }
                    } else {
                        // Move Left
                        if current_p_x > 0 {
                            if let Some(c) = line[..current_p_x].chars().next_back() {
                                self.cursor.x -= c.len_utf8() as u16;
                                self.cursor.target_x = self.calculate_visual_x(current_p_y, self.cursor.x as usize, info);
                            }
                        }
                    }
                }
            }
        } else {
            self.cursor.move_rel(Some(dx), Some(dy), &self.content, mode)?;
        }
        self.needs_update = true;
        Ok(())
    }

    fn calculate_visual_x(&self, p_y: usize, p_x: usize, info: &[(usize, usize, usize, bool)]) -> u16 {
        if let Some(segment) = info.iter().filter(|i| i.0 == p_y && i.1 <= p_x).last() {
            let line = &self.content[p_y];
            let mut vw = 0;
            for c in line[segment.1..p_x].chars() {
                use unicode_width::UnicodeWidthStr;
                vw += UnicodeWidthStr::width(c.to_string().as_str());
            }
            vw as u16
        } else {
            0
        }
    }

    pub fn is_expired(&self) -> bool {
        self.timer.is_some_and(|t| t.is_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_component_highlights() {
        let config = Config::default();
        let mut highlighter = SyntaxHighlighter::new();
        let content = vec!["fn main() {".to_string(), "    let x = 5;".to_string(), "}".to_string()];
        let mut comp = Component::new(content, ComponentType::Buffer, Some("main.rs".to_string()), &config);
        
        comp.update(Duration::from_millis(0), &config, &mut highlighter).unwrap();
        
        // Check that highlights were populated
        assert_eq!(comp.highlights.len(), 3);
        // "fn" is in the first line
        assert_eq!(comp.highlights[0][0], Some(Color::Magenta)); // 'f'
        assert_eq!(comp.highlights[0][1], Some(Color::Magenta)); // 'n'
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
            component.needs_update = true;
            component.modified = true;
        }
        TextActions::Delete => {
            let x = component.cursor.x as usize;
            let y = component.cursor.y as usize;

            if x > 0 {
                // Remove character before cursor
                component.content[y].remove(x - 1);
                component.cursor.x -= 1;
                component.needs_update = true;
                component.modified = true;
            } else if y > 0 {
                // Join with previous line
                let current_line = component.content.remove(y);
                let prev_line = &mut component.content[y - 1];
                let prev_len = prev_line.len() as u16;
                prev_line.push_str(&current_line);
                component.cursor.y -= 1;
                component.cursor.x = prev_len;
                component.needs_update = true;
                component.modified = true;
            }
        }
        TextActions::Insert(c) => {
            component.content[component.cursor.y as usize].insert(component.cursor.x as usize, *c);
            component
                .cursor
                .move_rel(Some(1), None, &component.content, mode)?;
            component.needs_update = true;
            component.modified = true;
        }
    }
    Ok(())
}
