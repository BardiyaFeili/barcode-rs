use std::error::Error;
use crossterm::{terminal, style::Color};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WindowType {
    Tile,
    Floating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum BorderStyle {
    None,
    Single,
    Double,
    Rounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum HorizontalAnchor {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum VerticalAnchor {
    Top,
    Center,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct WindowColors {
    pub border_fg: Color,
    pub border_bg: Option<Color>,
    pub content_fg: Color,
    pub content_bg: Option<Color>,
}

impl Default for WindowColors {
    fn default() -> Self {
        Self {
            border_fg: Color::White,
            border_bg: None,
            content_fg: Color::Reset,
            content_bg: None,
        }
    }
}

#[derive(Debug)]
pub struct Window {
    pub content: Vec<String>,
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub window_width: u16,
    pub window_height: u16,
    pub flexible_x: bool,
    pub flexible_y: bool,
    pub window_type: WindowType,
    pub h_anchor: HorizontalAnchor,
    pub v_anchor: VerticalAnchor,
    pub hidden: bool,
    pub viewpoint: usize,
    // Floating specific
    pub x: u16,
    pub y: u16,
    pub border_style: BorderStyle,
    pub colors: WindowColors,
}

impl Window {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        content: Vec<String>,
        width: Option<u16>,
        height: Option<u16>,
        flexible_x: bool,
        flexible_y: bool,
        window_type: WindowType,
        h_anchor: HorizontalAnchor,
        v_anchor: VerticalAnchor,
        hidden: bool,
        viewpoint: usize,
    ) -> Result<Self, Box<dyn Error>> {
        let (mut terminal_width, mut terminal_height) = terminal::size()?;
        if !flexible_x {
            terminal_width = width.unwrap_or(0);
        }
        if !flexible_y {
            terminal_height = height.unwrap_or(0);
        }
        Ok(Self {
            content,
            width,
            height,
            window_width: terminal_width,
            window_height: terminal_height,
            flexible_x,
            flexible_y,
            window_type,
            h_anchor,
            v_anchor,
            hidden,
            viewpoint,
            x: 0,
            y: 0,
            border_style: BorderStyle::None,
            colors: WindowColors::default(),
        })
    }

    pub fn set_floating(&mut self, x: u16, y: u16, width: u16, height: u16, border: BorderStyle) {
        self.window_type = WindowType::Floating;
        self.x = x;
        self.y = y;
        self.window_width = width;
        self.window_height = height;
        self.border_style = border;
    }

    pub fn calculate_absolute_pos(&self, term_w: u16, term_h: u16) -> (u16, u16) {
        if self.window_type == WindowType::Tile {
            return (0, 0); // Tiled position is handled by render logic
        }

        let x = match self.h_anchor {
            HorizontalAnchor::Left => self.x,
            HorizontalAnchor::Center => (term_w.saturating_sub(self.window_width)) / 2,
            HorizontalAnchor::Right => term_w.saturating_sub(self.window_width + self.x),
        };

        let y = match self.v_anchor {
            VerticalAnchor::Top => self.y,
            VerticalAnchor::Center => (term_h.saturating_sub(self.window_height)) / 2,
            VerticalAnchor::Bottom => term_h.saturating_sub(self.window_height + self.y),
        };

        (x, y)
    }
}
