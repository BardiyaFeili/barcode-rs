use std::{
    error::Error,
    io::{BufWriter, Write, stdout},
};

use crossterm::{
    cursor::{self, SetCursorStyle},
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use unicode_width::UnicodeWidthStr;

use crate::{
    component::{Component, ComponentType},
    config::Config,
    modal::Mode,
    window::{BorderStyle, VerticalAnchor, WindowType},
};

struct DrawContext<'a> {
    config: &'a Config,
    mode: &'a Mode,
    active_comp_info: Option<(u16, u16, u16, u16, usize)>,
    focused_comp_ptr: Option<*const Component>,
    is_active_input: bool,
}

pub fn render(
    active_components: &mut [Component],
    mode: &Mode,
    focused_idx: usize,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let (terminal_width, terminal_height) = terminal::size()?;

    let mut tiled_center = Vec::new();
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
            WindowType::Tile => {
                if component.window.v_anchor == VerticalAnchor::Center {
                    tiled_center.push(component);
                }
            }
            WindowType::Floating => floating.push(component),
        }
    }

    let mut active_comp_info = None;

    let mut current_min_x: u16 = 0;
    for component in &tiled_center {
        if active_comp_ptr.is_some_and(|ptr| std::ptr::eq(*component as *const Component, ptr)) {
            let (cx, cy) = component
                .window
                .visual_cursor
                .unwrap_or((component.cursor.x, component.cursor.y));
            // Tiled windows start at 0,0 relative to their assigned area.
            active_comp_info = Some((current_min_x, 0, cx, cy, component.window.viewpoint));
        }
        current_min_x = current_min_x.saturating_add(component.window.window_width);
    }

    // Floating Layout Calculation & Active check
    for component in floating.iter_mut() {
        if active_comp_ptr.is_some_and(|ptr| std::ptr::eq(*component as *const Component, ptr)) {
            let (abs_x, abs_y) = component
                .window
                .calculate_absolute_pos(terminal_width, terminal_height);
            let border_offset = if component.window.border_style != BorderStyle::None {
                1
            } else {
                0
            };
            let (cx, cy) = component
                .window
                .visual_cursor
                .unwrap_or((component.cursor.x, component.cursor.y));

            // Adjust coordinates by border offset for cursor positioning
            active_comp_info = Some((
                abs_x + border_offset,
                abs_y + border_offset,
                cx,
                cy,
                component.window.viewpoint,
            ));
        }
    }

    let ctx = DrawContext {
        config,
        mode,
        active_comp_info,
        focused_comp_ptr: active_comp_ptr,
        is_active_input,
    };

    draw(tiled_center, floating, status_line, ctx)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn queue_box(
    stdout: &mut BufWriter<std::io::Stdout>,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    style: BorderStyle,
    fg: Color,
    bg: Option<Color>,
    term_w: u16,
    term_h: u16,
) -> Result<(), Box<dyn Error>> {
    if style == BorderStyle::None || width < 2 || height < 2 {
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

    let inner_w = width.saturating_sub(2) as usize;

    if y < term_h {
        queue!(stdout, cursor::MoveTo(x, y))?;
        if x < term_w {
            queue!(stdout, Print(tl))?;
            if inner_w > 0 {
                let h_len = inner_w.min((term_w.saturating_sub(x + 1)) as usize);
                queue!(stdout, Print(h.repeat(h_len)))?;
            }
            if x + width - 1 < term_w {
                queue!(stdout, Print(tr))?;
            }
        }
    }

    for i in 1..height.saturating_sub(1) {
        if y + i < term_h {
            if x < term_w {
                queue!(stdout, cursor::MoveTo(x, y + i), Print(v))?;
            }
            if x + width - 1 < term_w {
                queue!(stdout, cursor::MoveTo(x + width - 1, y + i), Print(v))?;
            }
        }
    }

    if y + height - 1 < term_h && height > 1 {
        queue!(stdout, cursor::MoveTo(x, y + height - 1))?;
        if x < term_w {
            queue!(stdout, Print(bl))?;
            if inner_w > 0 {
                let h_len = inner_w.min((term_w.saturating_sub(x + 1)) as usize);
                queue!(stdout, Print(h.repeat(h_len)))?;
            }
            if x + width - 1 < term_w {
                queue!(stdout, Print(br))?;
            }
        }
    }

    queue!(stdout, ResetColor)?;
    Ok(())
}

fn draw(
    tiled_center: Vec<&mut Component>,
    floating: Vec<&mut Component>,
    status_line: Option<&mut Component>,
    ctx: DrawContext,
) -> Result<(), Box<dyn Error>> {
    let mut stdout = BufWriter::new(stdout());
    let (terminal_width, terminal_height) = terminal::size()?;

    queue!(stdout, cursor::Hide,)?;

    // Draw Tiled
    let mut current_x: u16 = 0;
    let tiled_count = tiled_center.len();
    let theme = &ctx.config.theme;

    // Pre-calculate focus for all tiled windows to handle shared borders correctly
    let focused_indices: Vec<bool> = tiled_center
        .iter()
        .map(|c| {
            ctx.focused_comp_ptr
                .is_some_and(|ptr| std::ptr::eq(*c as *const _, ptr))
        })
        .collect();

    for (idx, component) in tiled_center.into_iter().enumerate() {
        let win_w = component.window.window_width;
        let win_h = component.window.window_height;
        let content_w = if idx < tiled_count - 1 {
            win_w.saturating_sub(1)
        } else {
            win_w
        };

        let bg = component.window.colors.content_bg.or(theme.bg);
        let fg = component.window.colors.content_fg.unwrap_or(theme.fg);

        if let Some(bg_color) = bg {
            queue!(stdout, SetBackgroundColor(bg_color))?;
        } else {
            queue!(stdout, SetBackgroundColor(Color::Reset))?;
        }
        queue!(stdout, SetForegroundColor(fg))?;

        for y in 0..win_h {
            if y >= terminal_height || current_x >= terminal_width {
                break;
            }
            queue!(stdout, cursor::MoveTo(current_x, y))?;

            let is_focused = focused_indices[idx];
            let visual_idx = component.window.viewpoint + y as usize;
            let mut is_current_line = false;

            if is_focused && ctx.config.editor.line_indicator {
                if let Some(info) = &component.window.visual_cursor_info {
                    if let Some(&(p_y, _, _, _)) = info.get(visual_idx)
                        && p_y == component.cursor.y as usize
                    {
                        is_current_line = true;
                    }
                } else if visual_idx == component.cursor.y as usize {
                    is_current_line = true;
                }
            }

            let gutter_w = component.window.gutter_width as usize;
            if let Some(info) = &component.window.visual_cursor_info
                && gutter_w > 0
            {
                let visual_idx = component.window.viewpoint + y as usize;
                if let Some(&(p_y, _, _, is_first)) = info.get(visual_idx) {
                    let is_current = p_y == component.cursor.y as usize;
                    let fg_color = if is_current {
                        ctx.config.theme.gutter_active_fg
                    } else {
                        ctx.config.theme.gutter_fg
                    };

                    let g_bg = if is_current_line {
                        ctx.config
                            .theme
                            .line_indicator_bg
                            .or(ctx.config.theme.gutter_bg)
                    } else {
                        ctx.config.theme.gutter_bg
                    };

                    if let Some(bg_c) = g_bg {
                        queue!(stdout, SetBackgroundColor(bg_c))?;
                    } else {
                        queue!(stdout, SetBackgroundColor(Color::Reset))?;
                    }

                    queue!(stdout, SetForegroundColor(fg_color))?;

                    let pad_left = ctx.config.line_number.padding_left;
                    let pad_right = ctx.config.line_number.padding_right;

                    if is_first {
                        let num = match ctx.config.line_number.mode {
                            crate::config::LineNumberMode::Absolute => (p_y + 1).to_string(),
                            crate::config::LineNumberMode::Relative => {
                                if is_current {
                                    (p_y + 1).to_string()
                                } else {
                                    // Standard vim-like relative numbers:
                                    // distance from current line
                                    p_y.abs_diff(component.cursor.y as usize).to_string()
                                }
                            }
                            _ => String::new(),
                        };

                        let digits_w = gutter_w.saturating_sub(pad_left).saturating_sub(pad_right);
                        let num_padding = digits_w.saturating_sub(num.len());

                        queue!(
                            stdout,
                            Print(" ".repeat(pad_left)),
                            Print(" ".repeat(num_padding)),
                            Print(num),
                            Print(" ".repeat(pad_right)),
                        )?;
                    } else {
                        queue!(stdout, Print(" ".repeat(gutter_w)))?;
                    }

                    // Restore background for content
                    if let Some(bg_color) = if is_current_line {
                        ctx.config.theme.line_indicator_bg.or(bg)
                    } else {
                        bg
                    } {
                        queue!(stdout, SetBackgroundColor(bg_color))?;
                    } else {
                        queue!(stdout, SetBackgroundColor(Color::Reset))?;
                    }

                    queue!(stdout, SetForegroundColor(fg))?;
                }
            }

            let available_row_w =
                (terminal_width - current_x).saturating_sub(gutter_w as u16) as usize;
            let row_w = (content_w as usize)
                .saturating_sub(gutter_w)
                .min(available_row_w);

            // Set background for text area
            if let Some(bg_color) = if is_current_line {
                ctx.config.theme.line_indicator_bg.or(bg)
            } else {
                bg
            } {
                queue!(stdout, SetBackgroundColor(bg_color))?;
            } else {
                queue!(stdout, SetBackgroundColor(Color::Reset))?;
            }

            if let Some(line) = component.window.content.get(y as usize) {
                let line_highlights = component.window.highlights.get(y as usize);
                let mut curr_w = 0;
                let mut current_color = fg;
                let mut buffer = String::new();

                queue!(stdout, SetForegroundColor(fg))?;

                for (char_idx, c) in line.chars().enumerate() {
                    let cw = UnicodeWidthStr::width(c.to_string().as_str());
                    if curr_w + cw > row_w {
                        break;
                    }

                    let target_color = line_highlights
                        .and_then(|h| h.get(char_idx).copied().flatten())
                        .unwrap_or(fg);

                    if target_color != current_color {
                        if !buffer.is_empty() {
                            queue!(stdout, Print(&buffer))?;
                            buffer.clear();
                        }
                        queue!(stdout, SetForegroundColor(target_color))?;
                        current_color = target_color;
                    }

                    buffer.push(c);
                    curr_w += cw;
                }

                if !buffer.is_empty() {
                    queue!(stdout, Print(&buffer))?;
                }

                queue!(stdout, SetForegroundColor(fg))?;
                if curr_w < row_w {
                    queue!(stdout, Print(" ".repeat(row_w - curr_w)))?;
                }
            } else {
                queue!(stdout, Print(" ".repeat(row_w)))?;
            }

            // Draw vertical border if not last component
            if idx < tiled_count - 1 && current_x + win_w - 1 < terminal_width {
                let border_color = if tiled_count == 2 {
                    // Two windows: split the border vertically
                    let is_top_half = y < win_h / 2;
                    let active_idx = if focused_indices[0] { 0 } else { 1 };

                    if (active_idx == 0 && is_top_half) || (active_idx == 1 && !is_top_half) {
                        theme.accent
                    } else {
                        theme.border
                    }
                } else {
                    // 3+ windows: highlight border if either adjacent window is active
                    if focused_indices[idx]
                        || (idx + 1 < focused_indices.len() && focused_indices[idx + 1])
                    {
                        theme.accent
                    } else {
                        theme.border
                    }
                };

                queue!(
                    stdout,
                    cursor::MoveTo(current_x + win_w - 1, y),
                    SetForegroundColor(border_color),
                    Print("│")
                )?;
                // Restore colors for next line/component
                if let Some(bg_color) = bg {
                    queue!(stdout, SetBackgroundColor(bg_color))?;
                } else {
                    queue!(stdout, SetBackgroundColor(Color::Reset))?;
                }
                queue!(stdout, SetForegroundColor(fg))?;
            }
        }
        current_x = current_x.saturating_add(win_w);
    }

    // Draw Status Line
    if let Some(status) = status_line {
        let bar_bg = theme.status_bg;
        let bar_fg = theme.status_fg;
        let y = terminal_height.saturating_sub(1);

        queue!(stdout, cursor::MoveTo(0, y))?;

        // 1. Draw Left End
        queue!(stdout, ResetColor, SetForegroundColor(bar_bg))?;
        if let Some(bg) = theme.bg {
            queue!(stdout, SetBackgroundColor(bg))?;
        }
        queue!(stdout, Print(&ctx.config.status_line.left_end))?;

        // 2. Draw Main Bar
        queue!(
            stdout,
            SetBackgroundColor(bar_bg),
            SetForegroundColor(bar_fg)
        )?;
        let line = &status.content[0];
        queue!(stdout, Print(line))?;

        // 3. Draw Right End
        queue!(stdout, ResetColor, SetForegroundColor(bar_bg))?;
        if let Some(bg) = theme.bg {
            queue!(stdout, SetBackgroundColor(bg))?;
        }
        queue!(stdout, Print(&ctx.config.status_line.right_end))?;

        queue!(stdout, ResetColor)?;

        // Cleanup any artifacts below the status line
        if y + 1 < terminal_height {
            queue!(
                stdout,
                cursor::MoveTo(0, y + 1),
                terminal::Clear(terminal::ClearType::FromCursorDown)
            )?;
        }
    }
    for component in floating {
        let (abs_x, abs_y) = component
            .window
            .calculate_absolute_pos(terminal_width, terminal_height);
        let win = &component.window;
        let win_w = win.window_width.min(terminal_width.saturating_sub(abs_x));
        let win_h = win.window_height.min(terminal_height.saturating_sub(abs_y));
        let has_border = win.border_style != BorderStyle::None;

        let is_focused = ctx
            .focused_comp_ptr
            .is_some_and(|ptr| std::ptr::eq(component as *const _, ptr));

        let bg = win.colors.content_bg.or(theme.bg);
        let fg = win.colors.content_fg.unwrap_or(theme.fg);
        let border_fg = win.colors.border_fg.unwrap_or(if is_focused {
            theme.accent
        } else {
            theme.border
        });
        let border_bg = win.colors.border_bg.or(theme.bg);

        if let Some(bg_color) = bg {
            queue!(stdout, SetBackgroundColor(bg_color))?;
        } else {
            queue!(stdout, SetBackgroundColor(Color::Reset))?;
        }
        for i in 0..win_h {
            queue!(
                stdout,
                cursor::MoveTo(abs_x, abs_y + i),
                Print(" ".repeat(win_w as usize))
            )?;
        }

        if has_border {
            queue_box(
                &mut stdout,
                abs_x,
                abs_y,
                win.window_width,
                win.window_height,
                win.border_style,
                border_fg,
                border_bg,
                terminal_width,
                terminal_height,
            )?;
        }

        let content_x = if has_border { abs_x + 1 } else { abs_x };
        let content_y = if has_border { abs_y + 1 } else { abs_y };
        let content_h = if has_border {
            win_h.saturating_sub(2)
        } else {
            win_h
        };
        let content_w = if has_border {
            win_w.saturating_sub(2)
        } else {
            win_w
        };

        queue!(stdout, SetForegroundColor(fg))?;
        if let Some(bg_color) = bg {
            queue!(stdout, SetBackgroundColor(bg_color))?;
        }

        for (i, line) in component
            .window
            .content
            .iter()
            .take(content_h as usize)
            .enumerate()
        {
            if content_y + (i as u16) >= terminal_height {
                break;
            }
            queue!(stdout, cursor::MoveTo(content_x, content_y + (i as u16)))?;

            let mut text = line.clone();
            if component.component_type == ComponentType::Input && component.prompt_action.is_none()
            {
                text = format!(":{}", line);
            }

            let mut display_line = String::new();
            let mut curr_w = 0;
            for c in text.chars() {
                let cw = UnicodeWidthStr::width(c.to_string().as_str());
                if curr_w + cw > content_w as usize {
                    break;
                }
                display_line.push(c);
                curr_w += cw;
            }
            queue!(stdout, Print(&display_line))?;
        }
        queue!(stdout, ResetColor)?;
    }

    // Draw Cursor
    if let Some((abs_x, abs_y, cursor_x, cursor_y, viewpoint)) = ctx.active_comp_info {
        let relative_y = cursor_y as i32 - viewpoint as i32;
        if relative_y >= 0 && (abs_y + relative_y as u16) < terminal_height {
            // Find the component to get its gutter width
            // This is a bit slow but safe for now.
            // In a cleaner refactor, active_comp_info should include gutter_width.

            let mut gutter_offset = 0;
            if let Some(focused_ptr) = ctx.focused_comp_ptr {
                unsafe {
                    // Safe because we know focused_ptr is valid during render
                    gutter_offset = (*focused_ptr).window.gutter_width;
                }
            }

            let mut final_x: u16 = abs_x + cursor_x + gutter_offset;
            if ctx.is_active_input {
                // Only add ':' offset if it's NOT a prompt
                let is_prompt = if let Some(focused_ptr) = ctx.focused_comp_ptr {
                    unsafe { (*focused_ptr).prompt_action.is_some() }
                } else {
                    false
                };
                if !is_prompt {
                    final_x = final_x.saturating_add(1);
                }
            }
            let final_y = abs_y + relative_y as u16;

            if final_x < terminal_width {
                let (cursor_style, cx, cy) = match ctx.mode {
                    Mode::Command => (SetCursorStyle::SteadyBar, final_x, final_y),
                    Mode::Insert => (SetCursorStyle::SteadyBar, final_x, final_y),
                    _ => (SetCursorStyle::SteadyBlock, final_x, final_y),
                };

                queue!(stdout, cursor_style, cursor::MoveTo(cx, cy), cursor::Show)?;
            }
        }
    }

    stdout.flush()?;
    Ok(())
}
