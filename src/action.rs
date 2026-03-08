use std::error::Error;

use crate::{
    command::handle_command,
    component::{Component, ComponentType},
    config::Config,
    input::handle_cursor_action,
    log::log,
    modal::Mode,
    window::{WindowType, handle_window_action, remove_component},
};

#[derive(Debug, PartialEq)]
pub enum Action {
    Text(TextActions),
    Cursor(CursorActions),
    Window(WindowActions),
    Mode(Mode),
    Command(String),
    ExecuteCommand(String),
    Prompt(PromptAction),
    ExecutePrompt(PromptAction, Option<String>),
    Quit,
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PromptAction {
    ConfirmQuit(usize),              // buffer index
    ConfirmSaveAs(usize, String),    // buffer index, path
    ConfirmCreateDir(usize, String), // buffer index, path
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
        let mut input_comp =
            Component::new(vec![String::new()], ComponentType::Input, None, config);
        let cfg = &config.input;
        input_comp.window.window_type = WindowType::Floating;
        input_comp.window.h_anchor = cfg.h_anchor;
        input_comp.window.v_anchor = cfg.v_anchor;
        input_comp.window.x = 0;
        input_comp.window.y = 1;
        input_comp.window.window_width = 80;
        input_comp.window.window_height = 3;
        input_comp.window.border_style = cfg.border_style;
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
                handle_cursor_action(Some(component), a, mode, config)?;
            }
        }
        Action::Mode(new_mode) => {
            if let Some(component) = active_components.get_mut(*focused_idx) {
                component
                    .cursor
                    .move_abs(None, None, &component.content, new_mode, None)?;
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
            let action = handle_command(cmd, active_components, focused_idx, config)?;
            if action != Action::None {
                return take_action(
                    &action,
                    focused_idx,
                    active_components,
                    mode,
                    old_mode,
                    config,
                );
            }
        }
        Action::Window(a) => {
            handle_window_action(a, focused_idx, active_components)?;
        }
        Action::Prompt(p) => {
            let msg = match &p {
                PromptAction::ConfirmQuit(_) => "Unsaved changes. Quit anyway? (y/n)".to_string(),
                PromptAction::ConfirmSaveAs(_, _) => "Save as: ".to_string(),
                PromptAction::ConfirmCreateDir(_, _) => {
                    "Directory does not exist. Create? (y/n)".to_string()
                }
            };

            let mut input_comp = Component::new(vec![msg], ComponentType::Input, None, config)
                .with_prompt_action(p.clone());

            let cfg = &config.input;
            input_comp.window.window_type = WindowType::Floating;
            input_comp.window.h_anchor = cfg.h_anchor;
            input_comp.window.v_anchor = cfg.v_anchor;
            input_comp.window.x = 0;
            input_comp.window.y = 1;
            input_comp.window.window_width = 80;
            input_comp.window.window_height = 3;
            input_comp.window.border_style = cfg.border_style;

            if let PromptAction::ConfirmSaveAs(_, _) = p {
                input_comp.cursor.x = input_comp.content[0].len() as u16;
                input_comp.cursor.target_x = input_comp.cursor.x;
            }

            active_components.push(input_comp);
            *focused_idx = active_components.len() - 1;
        }
        Action::ExecutePrompt(p, response) => {
            handle_execute_prompt(
                p.clone(),
                response.clone(),
                active_components,
                focused_idx,
                config,
            )?;
        }
        _ => (),
    }
    Ok(())
}

fn handle_execute_prompt(
    p: PromptAction,
    response: Option<String>,
    active_components: &mut Vec<Component>,
    focused_idx: &mut usize,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    // Remove the input component
    remove_input_component(active_components, focused_idx);

    let r = response.unwrap_or_default().to_lowercase();
    match p {
        PromptAction::ConfirmQuit(idx) => {
            if r == "y" {
                remove_component(active_components, focused_idx, idx);
            }
        }
        PromptAction::ConfirmSaveAs(idx, _) => {
            if !r.is_empty() && r != "n" {
                let path = r;
                if !crate::file::parent_exists(&path) {
                    // Chain a second prompt for directory creation
                    let mut input_comp = Component::new(
                        vec!["Directory does not exist. Create? (y/n)".to_string()],
                        ComponentType::Input,
                        None,
                        config,
                    )
                    .with_prompt_action(PromptAction::ConfirmCreateDir(idx, path));

                    let cfg = &config.input;
                    input_comp.window.window_type = WindowType::Floating;
                    input_comp.window.h_anchor = cfg.h_anchor;
                    input_comp.window.v_anchor = cfg.v_anchor;
                    input_comp.window.x = 0;
                    input_comp.window.y = 1;
                    input_comp.window.window_width = 80;
                    input_comp.window.window_height = 3;
                    input_comp.window.border_style = cfg.border_style;

                    active_components.push(input_comp);
                    *focused_idx = active_components.len() - 1;
                } else if let Some(comp) = active_components.get_mut(idx) {
                    comp.file_path = Some(path.clone());
                    crate::file::save_file(&path, &comp.content)?;
                    comp.modified = false;
                    crate::notification::push_notification(
                        active_components,
                        format!("Saved {}", path),
                        config,
                    )?;
                }
            }
        }
        PromptAction::ConfirmCreateDir(idx, path) => {
            if let Some(comp) = active_components.get_mut(idx)
                && r == "y"
            {
                comp.file_path = Some(path.clone());
                crate::file::save_file(&path, &comp.content)?;
                comp.modified = false;
                crate::notification::push_notification(
                    active_components,
                    format!("Saved {}", path),
                    config,
                )?;
            }
        }
    }
    Ok(())
}

fn remove_input_component(active_components: &mut Vec<Component>, focused_idx: &mut usize) {
    if let Some(pos) = active_components
        .iter()
        .position(|c| c.component_type == ComponentType::Input)
    {
        remove_component(active_components, focused_idx, pos);
    }
}
