use std::error::Error;
use crossterm::{terminal, style::Color};
use serde::Deserialize;

use crate::{action::WindowActions, component::{Component, ComponentType}};

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

#[derive(Debug, Clone, Copy, Default)]
pub struct WindowColors {
    pub border_fg: Option<Color>,
    pub border_bg: Option<Color>,
    pub content_fg: Option<Color>,
    pub content_bg: Option<Color>,
}

#[derive(Debug)]
pub struct Window {
    pub content: Vec<String>,
    pub highlights: Vec<Vec<Option<Color>>>,
    pub width: Option<u16>,
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
    pub gutter_width: u16,
    pub visual_cursor: Option<(u16, u16)>,
    pub visual_cursor_info: Option<Vec<(usize, usize, usize, bool)>>, // (physical_y, physical_x, visual_y, is_first)
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
            highlights: Vec::new(),
            width,
            window_width: terminal_width,
            window_height: terminal_height,
            flexible_x,
            flexible_y,
            window_type,
            h_anchor,
            v_anchor,
            hidden,
            viewpoint,
            gutter_width: 0,
            visual_cursor: None,
            visual_cursor_info: None,
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

pub fn recalculate_layouts(components: &mut [Component]) -> Result<(), Box<dyn Error>> {
    let (terminal_width, terminal_height) = terminal::size()?;

    let mut tiled_center = Vec::new();
    let mut status_line_exists = false;
    
    // For now we only handle Center tiled windows and StatusLine
    for component in components.iter_mut() {
        if component.window.hidden { continue; }
        if component.component_type == ComponentType::StatusLine {
            status_line_exists = true;
            continue;
        }
        if component.window.window_type == WindowType::Tile && component.window.v_anchor == VerticalAnchor::Center {
            tiled_center.push(component);
        }
    }

    let mut center_height = terminal_height;
    if status_line_exists {
        center_height = center_height.saturating_sub(1);
    }

    let mut center_flexible_width = terminal_width;
    for component in &tiled_center {
        if !component.window.flexible_x {
            center_flexible_width = center_flexible_width.saturating_sub(component.window.width.unwrap_or(0));
        }
    }

    let center_flexible_window_width = if tiled_center.is_empty() {
        center_flexible_width
    } else {
        let flex_count = tiled_center.iter().filter(|c| c.window.flexible_x).count();
        if flex_count > 0 {
            center_flexible_width / flex_count as u16
        } else {
            0
        }
    };

    let mut distributed_width = 0;
    let flex_indices: Vec<usize> = tiled_center.iter().enumerate()
        .filter(|(_, c)| c.window.flexible_x)
        .map(|(i, _)| i)
        .collect();

    for (idx, component) in tiled_center.iter_mut().enumerate() {
        component.window.window_height = center_height;
        let old_w = component.window.window_width;
        let old_h = component.window.window_height;

        if component.window.flexible_x {
            if Some(&idx) == flex_indices.last() {
                // Last flexible window gets the remaining width
                component.window.window_width = center_flexible_width.saturating_sub(distributed_width);
            } else {
                component.window.window_width = center_flexible_window_width;
                distributed_width = distributed_width.saturating_add(center_flexible_window_width);
            }
        } else {
            component.window.window_width = component.window.width.unwrap_or(0);
            distributed_width = distributed_width.saturating_add(component.window.window_width);
        }

        if old_w != component.window.window_width || old_h != component.window.window_height {
            component.needs_update = true;
        }
    }

    Ok(())
}

pub fn remove_component(components: &mut Vec<Component>, focused_idx: &mut usize, target_idx: usize) {
    if target_idx < components.len() {
        components.remove(target_idx);
        
        if components.is_empty() {
            *focused_idx = 0;
            return;
        }

        // Adjust focused_idx if it was pointing at or after the removed component
        if *focused_idx >= target_idx && *focused_idx > 0 {
            *focused_idx -= 1;
        }

        // Cap focused_idx at max index
        if *focused_idx >= components.len() {
            *focused_idx = components.len() - 1;
        }

        // Ensure focused index is focusable. 
        // When we remove a component (especially the command bar), 
        // we usually want to go BACKWARDS to find the last focused buffer.
        let start_idx = *focused_idx;
        while !components[*focused_idx].focusable {
            if *focused_idx > 0 {
                *focused_idx -= 1;
            } else {
                // If we hit 0 and it's not focusable, look forwards
                let mut found = false;
                for (i, c) in components.iter().enumerate() {
                    if c.focusable {
                        *focused_idx = i;
                        found = true;
                        break;
                    }
                }
                if !found {
                    *focused_idx = 0; // No focusable components
                }
                break;
            }
            if *focused_idx == start_idx {
                break;
            }
        }
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
