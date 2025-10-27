use std::error::Error;

use crossterm::terminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowType {
    Tile,
    Floating,
}

#[derive(Debug)]
pub struct Window {
    pub content: Vec<String>, // lines inside the window
    pub width: Option<u16>,   // None = flexible
    pub height: Option<u16>,  // None = flexible
    pub window_width: u16,
    pub window_height: u16,
    pub flexible_x: bool,        // true if width can expand
    pub flexible_y: bool,        // true if height can expand
    pub window_type: WindowType, // tile or floating
    pub position: Position,
    pub hidden: bool
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Top,
    Center,
    Bottom,
}

impl Window {
    /// Creates a new window object
    pub fn new(
        content: Vec<String>,
        width: Option<u16>,
        height: Option<u16>,
        flexible_x: bool,
        flexible_y: bool,
        window_type: WindowType,
        position: Position,
        hidden: bool
    ) -> Result<Self, Box<dyn Error>> {
        let (mut terminal_width, mut terminal_height) = terminal::size()?;
        if !flexible_x {
            terminal_width = width.unwrap_or(0) as u16;
        }
        if !flexible_y {
            terminal_height = width.unwrap_or(0) as u16;
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
            position,
            hidden
        })
    }
}
