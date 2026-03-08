use std::{
    error::Error,
    io::{Write, stdout},
};

use crossterm::{
    cursor::{self, SetCursorStyle}, queue,
    terminal::{self, Clear, ClearType},
    style::{Color, Print, SetForegroundColor, SetBackgroundColor, ResetColor},
};

use crate::{
    component::{Component, ComponentType},
    modal::Mode,
    window::{WindowType, BorderStyle, VerticalAnchor},
    config::Config,
};

pub fn render(active_components: &mut [Component], mode: &Mode, focused_idx: usize, config: &Config) -> Result<(), Box<dyn Error>> {
    let (terminal_width, terminal_height) = terminal::size()?;

    let (mut tiled_top, mut tiled_center, mut tiled_bottom) =
        (Vec::new(), Vec::new(), Vec::new());
    let mut floating = Vec::new();
    let mut status_line = None;
    
    // We need to keep track of which component is the active one
    let mut active_comp_ptr: Option<*const Component> = None;
    let mut is_active_input = false;
    if focused_idx < active_components.len() {
        let active = &active_components[focused_idx];
        active_comp_ptr = Some(active as *const Component);
        is_active_input = active.component_type == ComponentType::Input;
    }

    for component in &mut *active_components {
        if component.window.hidden {
            continue;
        }
        if component.component_type == ComponentType::StatusLine {
            status_line = Some(component);
            continue;
        }

        match component.window.window_type {
            WindowType::Tile => match component.window.v_anchor {
                VerticalAnchor::Top => tiled_top.push(component),
                VerticalAnchor::Center => tiled_center.push(component),
                VerticalAnchor::Bottom => tiled_bottom.push(component),
            },
            WindowType::Floating => floating.push(component),
        }
    }

    // Tiled Layout Calculation
    let mut center_height = terminal_height;
    
    // Subtract status line height if it exists
    if status_line.is_some() {
        center_height -= 1;
    }

    for component in tiled_top.iter().chain(tiled_bottom.iter()) {
        center_height -= component.window.height.unwrap_or(0);
    }

    let mut center_flexible_width = terminal_width;
    for component in &tiled_center {
        if !component.window.flexible_x {
            center_flexible_width -= component.window.width.unwrap_or(0);
        }
    }

    let center_flexible_window_width = if tiled_center.is_empty() {
        center_flexible_width
    } else {
        center_flexible_width / tiled_center.len() as u16
    };

    let mut active_comp_info = None;

    let mut current_min_x = 0;
    for component in tiled_center.iter_mut() {
        component.window.window_height = center_height;
        if component.window.flexible_x {
            component.window.window_width = center_flexible_window_width;
        }
        
        if active_comp_ptr.is_some_and(|ptr| std::ptr::eq(*component, ptr)) {
            // Tiled windows start at 0,0 relative to their assigned area.
            active_comp_info = Some((current_min_x, 0, component.cursor.x, component.cursor.y, component.window.viewpoint));
        }
        current_min_x += component.window.window_width;
    }

    // Floating Layout Calculation & Active check
    for component in floating.iter_mut() {
         if active_comp_ptr.is_some_and(|ptr| std::ptr::eq(*component, ptr)) {
            let (abs_x, abs_y) = component.window.calculate_absolute_pos(terminal_width, terminal_height);
            let border_offset = if component.window.border_style != BorderStyle::None { 1 } else { 0 };
            active_comp_info = Some((abs_x + border_offset, abs_y + border_offset, component.cursor.x, component.cursor.y, component.window.viewpoint));
        }
    }

    draw(tiled_center, floating, status_line, mode, active_comp_info, is_active_input, config)?;

    Ok(())
}

fn queue_box(
    stdout: &mut std::io::Stdout,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderStyle,
    fg: Color,
    bg: Option<Color>,
) -> Result<(), Box<dyn Error>> {
    if style == BorderStyle::None {
        return Ok(());
    }

    let (tl, tr, bl, br, h, v) = match style {
        BorderStyle::Single => ("┌", "┐", "└", "┘", "─", "│"),
        BorderStyle::Double => ("╔", "╗", "╚", "╝", "═", "║"),
        BorderStyle::Rounded => ("╭", "╮", "╰", "╯", "─", "│"),
        BorderStyle::None => unreachable!(),
    };

    queue!(stdout, SetForegroundColor(fg))?;
    if let Some(bg_color) = bg {
        queue!(stdout, SetBackgroundColor(bg_color))?;
    }

    queue!(stdout, cursor::MoveTo(x, y), Print(tl), Print(h.repeat((width.saturating_sub(2)) as usize)), Print(tr))?;
    for i in 1..height.saturating_sub(1) {
        queue!(stdout, cursor::MoveTo(x, y + i), Print(v), cursor::MoveTo(x + width - 1, y + i), Print(v))?;
    }
    queue!(stdout, cursor::MoveTo(x, y + height.saturating_sub(1)), Print(bl), Print(h.repeat((width.saturating_sub(2)) as usize)), Print(br))?;

    queue!(stdout, ResetColor)?;
    Ok(())
}

pub fn draw(
    tiled_center: Vec<&mut Component>,
    floating: Vec<&mut Component>,
    status_line: Option<&mut Component>,
    mode: &Mode,
    active_comp_info: Option<(u16, u16, u16, u16, usize)>,
    is_active_input: bool,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    let (terminal_width, terminal_height) = terminal::size()?;

    // Efficient clear: only from top-left once
    queue!(
        stdout,
        cursor::Hide,
        cursor::MoveTo(0, 0),
        Clear(ClearType::FromCursorDown)
    )?;

    // Draw Tiled
    let mut current_x = 0;
    let tiled_count = tiled_center.len();
    for (idx, component) in tiled_center.into_iter().enumerate() {
        let win_w = component.window.window_width;
        let win_h = component.window.window_height;
        let content_w = if idx < tiled_count - 1 { win_w.saturating_sub(1) } else { win_w };
        
        for y in 0..win_h {
            queue!(stdout, cursor::MoveTo(current_x, y))?;
            if let Some(line) = component.window.content.get(y as usize) {
                let truncated_line = if line.len() > content_w as usize {
                    &line[..content_w as usize]
                } else {
                    line
                };
                queue!(stdout, Print(truncated_line))?;
                // Fill rest of the line width with spaces to overwrite old content
                if truncated_line.len() < content_w as usize {
                    queue!(stdout, Print(" ".repeat(content_w as usize - truncated_line.len())))?;
                }
            } else {
                queue!(stdout, Print(" ".repeat(content_w as usize)))?;
            }
            
            // Draw vertical border if not last component
            if idx < tiled_count - 1 {
                queue!(stdout, cursor::MoveTo(current_x + win_w - 1, y), SetForegroundColor(Color::DarkGrey), Print("│"), ResetColor)?;
            }
        }
        current_x += win_w;
    }

    // Draw Status Line
    if let Some(status) = status_line {
        let bar_bg = Color::White;
        let bar_fg = Color::Black;
        
        queue!(stdout, cursor::MoveTo(0, terminal_height - 1))?;
        
        // 1. Draw Left End
        queue!(stdout, ResetColor, SetForegroundColor(bar_bg))?;
        queue!(stdout, Print(&config.status_line.left_end))?;
        
        // 2. Draw Main Bar
        queue!(stdout, SetBackgroundColor(bar_bg), SetForegroundColor(bar_fg))?;
        let line = &status.content[0];
        queue!(stdout, Print(line))?;
        
        // 3. Draw Right End
        queue!(stdout, ResetColor, SetForegroundColor(bar_bg))?;
        queue!(stdout, Print(&config.status_line.right_end))?;
        
        queue!(stdout, ResetColor)?;
    }

    // Draw Floating
    for component in floating {
        let (abs_x, abs_y) = component.window.calculate_absolute_pos(terminal_width, terminal_height);
        let win = &component.window;
        let has_border = win.border_style != BorderStyle::None;
        
        if let Some(bg) = win.colors.content_bg {
            queue!(stdout, SetBackgroundColor(bg))?;
        } else {
            queue!(stdout, SetBackgroundColor(Color::Reset))?;
        }
        for i in 0..win.window_height {
            queue!(stdout, cursor::MoveTo(abs_x, abs_y + i), Print(" ".repeat(win.window_width as usize)))?;
        }

        if has_border {
            queue_box(&mut stdout, abs_x, abs_y, win.window_width, win.window_height, win.border_style, win.colors.border_fg, win.colors.border_bg)?;
        }

        let content_x = if has_border { abs_x + 1 } else { abs_x };
        let content_y = if has_border { abs_y + 1 } else { abs_y };
        let content_height = if has_border { win.window_height.saturating_sub(2) } else { win.window_height };
        let content_width = if has_border { win.window_width.saturating_sub(2) } else { win.window_width };

        queue!(stdout, SetForegroundColor(win.colors.content_fg))?;
        if let Some(bg) = win.colors.content_bg {
            queue!(stdout, SetBackgroundColor(bg))?;
        }

        for (i, line) in component.window.content.iter().take(content_height as usize).enumerate() {
            queue!(stdout, cursor::MoveTo(content_x, content_y + i as u16))?;
            
            let mut display_line = line.clone();
            if component.component_type == ComponentType::Input {
                display_line = format!(":{}", line);
            }
            
            let truncated_line = if display_line.len() > content_width as usize {
                &display_line[..content_width as usize]
            } else {
                &display_line
            };
            queue!(stdout, Print(truncated_line))?;
        }
        queue!(stdout, ResetColor)?;
    }

    // Draw Cursor
    if let Some((abs_x, abs_y, cursor_x, cursor_y, viewpoint)) = active_comp_info {
        let relative_y = cursor_y as i32 - viewpoint as i32;
        if relative_y >= 0 {
             let mut final_x = abs_x + cursor_x;
             let final_y = abs_y + relative_y as u16;

             if is_active_input {
                 final_x += 1;
             }

            let (cursor_style, cx, cy) = match mode {
                Mode::Command => (SetCursorStyle::SteadyBar, final_x, final_y),
                Mode::Insert => (SetCursorStyle::SteadyBar, final_x, final_y),
                _ => (SetCursorStyle::SteadyBlock, final_x, final_y),
            };
            
            queue!(
                stdout,
                cursor_style,
                cursor::MoveTo(cx, cy),
                cursor::Show
            )?;
        }
    }

    stdout.flush()?;
    Ok(())
}
