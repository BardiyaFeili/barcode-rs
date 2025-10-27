use std::{error::Error, io::{stdout, Write}};

use crossterm::{cursor, execute, terminal::{self, Clear, ClearType}};

use crate::{component::Component, window::{Position, WindowType}};

pub fn render(active_components: &mut Vec<Component>) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;

    let (mut top_components, mut center_components, mut bottom_components) =
        (Vec::new(), Vec::new(), Vec::new());
    for component in active_components {
        match component.window.window_type {
            WindowType::Tile => match component.window.position {
                Position::Top => top_components.push(component),
                Position::Center => center_components.push(component),
                Position::Bottom => bottom_components.push(component),
            },
            WindowType::Floating => {}
        }
    }

    let mut center_height = height;
    for component in  top_components.iter().chain(bottom_components.iter()) {
        center_height -= component.window.height.unwrap_or_else(|| 0);
    }

    let mut center_flexible_width = width;
    for component in &center_components {
        if !component.window.flexible_x {
            center_flexible_width -= component.window.width.unwrap_or_else(|| 0);
        }
    }

    let center_flexible_window_width = center_flexible_width / center_components.len() as u16;

    for component in center_components.iter_mut() {
        component.window.window_height = center_height;
        if component.window.flexible_x {
            component.window.window_width = center_flexible_window_width;
        }
    }
    
    draw(center_components)?;
    
    Ok(())
}

pub fn draw(center_components: Vec<&mut Component>) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    execute!(
        stdout,
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown)
    )?;

    let mut min_x = 0;
    for component in center_components{
        for (y, line) in component.window.content.iter().enumerate(){
            execute!(stdout, cursor::MoveTo(min_x, y as u16))?;
            writeln!(stdout, "{}", line)?;
        }
        min_x += component.window.window_width
    }

    stdout.flush()?;
    Ok(())
}