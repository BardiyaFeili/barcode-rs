use std::{error::Error, io::{stdout, Cursor, Stdout, Write}};

use crossterm::{cursor, execute, terminal::{self, Clear, ClearType}};

use crate::window::{Position, Window, WindowType};

pub fn render(active_windows: &mut Vec<Window>) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;

    let (mut top_windows, mut center_windows, mut bottom_windows) =
        (Vec::new(), Vec::new(), Vec::new());
    for window in active_windows {
        match window.window_type {
            WindowType::Tile => match window.position {
                Position::Top => top_windows.push(window),
                Position::Center => center_windows.push(window),
                Position::Bottom => bottom_windows.push(window),
            },
            WindowType::Floating => {}
        }
    }

    let mut center_height = height;
    for window in top_windows.iter().chain(bottom_windows.iter()) {
        center_height -= window.height.unwrap_or_else(|| 0);
    }

    let mut center_flexible_width = width;
    for window in &center_windows {
        if !window.flexible_x {
            center_flexible_width -= window.width.unwrap_or_else(|| 0);
        }
    }

    let center_flexible_window_width = center_flexible_width / center_windows.len() as u16;

    for window in center_windows.iter_mut() {
        window.window_height = center_height;
        if window.flexible_x {
            window.window_width = center_flexible_window_width;
        }
    }
    
    draw(center_windows);
    
    Ok(())
}

pub fn draw(center_windows: Vec<&mut Window>) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown)
    )?;

    let mut min_x = 0;
    for window in center_windows{
        for (y, line) in window.content.iter().enumerate(){
            execute!(stdout, cursor::MoveTo(min_x, y as u16))?;
            writeln!(stdout, "{}", line)?;
        }
        min_x += window.window_width
    }

    stdout.flush()?;
    Ok(())
}