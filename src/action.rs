use std::error::Error;

use crate::{
    command::handle_command,
    component::{Component, ComponentType},
    config::Config,
    input::handle_cursor_action,
    log::log,
    modal::Mode,
    window::{handle_window_action, WindowType},
};

#[derive(Debug, PartialEq)]
pub enum Action {
    Text(TextActions),
    Cursor(CursorActions),
    Window(WindowActions),
    Mode(Mode),
    Command(String),
    ExecuteCommand(String),
    Quit,
    None,
}

#[derive(Debug, PartialEq)]
pub enum TextActions {
    NewLine,
    Insert(char),
    Delete,
}

#[derive(Debug, PartialEq)]
pub enum CursorActions {
    #[allow(dead_code)]
    MoveAbs(u16, u16),
    MoveRel(i16, i16),
}

#[derive(Debug, PartialEq)]
pub enum WindowActions {
    Next,
    #[allow(dead_code)]
    Previous,
    #[allow(dead_code)]
    Focus(usize),
}

pub fn take_action(
    action: &Action,
    focused_idx: &mut usize,
    active_components: &mut Vec<Component>,
    mode: &Mode,
    old_mode: Mode,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    if action != &Action::None {
        match action {
            Action::Cursor(_) | Action::Text(_) => {} // Too noisy
            _ => log(format!("Action: {:?}", action))?,
        }
    }

    // UI management based on mode transitions
    if *mode == Mode::Command && old_mode != Mode::Command {
        // Entering command mode: create input component
        let mut input_comp = Component::new(vec![String::new()], ComponentType::Input, None);
        let cfg = &config.input;
        input_comp.window.window_type = WindowType::Floating;
        input_comp.window.h_anchor = cfg.h_anchor;
        input_comp.window.v_anchor = cfg.v_anchor;
        input_comp.window.x = 0;
        input_comp.window.y = 1;
        input_comp.window.window_width = 80;
        input_comp.window.window_height = 3;
        input_comp.window.border_style = cfg.border_style;
        input_comp.window.colors.border_fg = crossterm::style::Color::Yellow;
        active_components.push(input_comp);
        *focused_idx = active_components.len() - 1;
    } else if *mode != Mode::Command && old_mode == Mode::Command {
        // Leaving command mode: remove input component
        remove_input_component(active_components, focused_idx);
    }

    match action {
        Action::Text(a) => {
            if let Some(component) = active_components.get_mut(*focused_idx) {
                crate::component::handle_write_action(Some(component), a, mode)?;
            }
        }
        Action::Cursor(a) => {
            if let Some(component) = active_components.get_mut(*focused_idx) {
                handle_cursor_action(Some(component), a, mode)?;
            }
        }
        Action::Mode(new_mode) => {
            if let Some(component) = active_components.get_mut(*focused_idx) {
                component
                    .cursor
                    .move_abs(None, None, &component.content, new_mode)?;
            }
        }
        Action::Command(cmd) => {
            if let Some(input_comp) = active_components
                .iter_mut()
                .find(|c| c.component_type == ComponentType::Input)
            {
                input_comp.content[0] = cmd.clone();
                input_comp.cursor.x = cmd.len() as u16;
                input_comp.needs_update = true;
            }
        }
        Action::ExecuteCommand(cmd) => {
            remove_input_component(active_components, focused_idx);
            handle_command(cmd, active_components, focused_idx, config)?;
        }
        Action::Window(a) => {
            handle_window_action(a, focused_idx, active_components)?;
        }
        _ => (),
    }
    Ok(())
}

fn remove_input_component(active_components: &mut Vec<Component>, focused_idx: &mut usize) {
    if let Some(pos) = active_components
        .iter()
        .position(|c| c.component_type == ComponentType::Input)
    {
        active_components.remove(pos);
        if *focused_idx >= active_components.len() && !active_components.is_empty() {
            *focused_idx = active_components.len() - 1;
        } else if active_components.is_empty() {
            *focused_idx = 0;
        }

        // Ensure focused index is focusable
        if !active_components.is_empty() {
            while !active_components[*focused_idx].focusable && *focused_idx > 0 {
                *focused_idx -= 1;
            }
        }
    }
}
