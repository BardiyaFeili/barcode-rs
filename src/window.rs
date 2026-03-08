use std::error::Error;
use crossterm::{terminal, style::Color};
use serde::Deserialize;

use crate::{action::WindowActions, component::Component};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum WindowType {
    Tile,
    Floating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BorderStyle {
    None,
    Single,
    Double,
    Rounded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HorizontalAnchor {
    #[serde(alias = "l")]
    Left,
    #[serde(alias = "c")]
    Center,
    #[serde(alias = "r")]
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAnchor {
    #[serde(alias = "t")]
    Top,
    #[serde(alias = "c")]
    Center,
    #[serde(alias = "b")]
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
            return (0, 0); 
        }

        // 1. Ensure window itself isn't larger than terminal
        let effective_w = self.window_width.min(term_w);
        let effective_h = self.window_height.min(term_h);

        // 2. Calculate anchored positions with clamping
        let x = match self.h_anchor {
            HorizontalAnchor::Left => self.x.min(term_w.saturating_sub(effective_w)),
            HorizontalAnchor::Center => (term_w.saturating_sub(effective_w)) / 2,
            HorizontalAnchor::Right => term_w.saturating_sub(effective_w.saturating_add(self.x)),
        };

        let y = match self.v_anchor {
            VerticalAnchor::Top => self.y.min(term_h.saturating_sub(effective_h)),
            VerticalAnchor::Center => (term_h.saturating_sub(effective_h)) / 2,
            VerticalAnchor::Bottom => term_h.saturating_sub(effective_h.saturating_add(self.y)),
        };

        (x, y)
    }
}

pub fn handle_window_action(
    action: &WindowActions,
    focused_idx: &mut usize,
    active_components: &[Component],
) -> Result<(), Box<dyn Error>> {
    match action {
        WindowActions::Next => {
            if !active_components.is_empty() {
                let mut next = (*focused_idx + 1) % active_components.len();
                let start = next;
                while !active_components[next].focusable {
                    next = (next + 1) % active_components.len();
                    if next == start {
                        break;
                    }
                }
                *focused_idx = next;
            }
        }
        WindowActions::Previous => {
            if !active_components.is_empty() {
                let mut prev = if *focused_idx == 0 {
                    active_components.len() - 1
                } else {
                    *focused_idx - 1
                };
                let start = prev;
                while !active_components[prev].focusable {
                    prev = if prev == 0 {
                        active_components.len() - 1
                    } else {
                        prev - 1
                    };
                    if prev == start {
                        break;
                    }
                }
                *focused_idx = prev;
            }
        }
        WindowActions::Focus(idx) => {
            if *idx < active_components.len() && active_components[*idx].focusable {
                *focused_idx = *idx;
            }
        }
    }
    Ok(())
}
