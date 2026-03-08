use std::error::Error;
use crate::{
    action::{Action, PromptAction},
    component::{Component, ComponentType},
    config::Config,
    file::{open_file, save_file, parent_exists},
    log::log,
    notification::push_notification,
    window::remove_component,
};

pub fn handle_command(
    cmd: &str,
    components: &mut Vec<Component>,
    focused_idx: &mut usize,
    config: &Config,
) -> Result<Action, Box<dyn Error>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(Action::None);
    }

    match parts[0] {
        "n" => {
            components.push(Component::new(
                vec![String::new()],
                ComponentType::Buffer,
                None,
                config,
            ));
            *focused_idx = components.len() - 1;
        }
        "w" => {
            if let Some(comp) = components.get(*focused_idx) {
                let path = if parts.len() > 1 {
                    Some(parts[1].to_string())
                } else {
                    comp.file_path.clone()
                };

                if let Some(path) = path {
                    if !parent_exists(&path) {
                        return Ok(Action::Prompt(PromptAction::ConfirmCreateDir(*focused_idx, path)));
                    }
                    save_file(&path, &comp.content)?;
                    log(format!("File saved: {}", path))?;
                    if let Some(comp_mut) = components.get_mut(*focused_idx) {
                        comp_mut.file_path = Some(path.clone());
                        comp_mut.modified = false;
                    }
                    push_notification(components, format!("Saved {}", path), config)?;
                } else {
                    return Ok(Action::Prompt(PromptAction::ConfirmSaveAs(*focused_idx, String::new())));
                }
            }
        }
        "q" => {
            if let Some(comp) = components.get(*focused_idx) {
                if comp.modified && comp.component_type == ComponentType::Buffer {
                    return Ok(Action::Prompt(PromptAction::ConfirmQuit(*focused_idx)));
                }
            }
            remove_component(components, focused_idx, *focused_idx);
        }
        "wq" => {
            if let Some(comp) = components.get(*focused_idx) {
                if let Some(path) = &comp.file_path {
                    if !parent_exists(path) {
                        return Ok(Action::Prompt(PromptAction::ConfirmCreateDir(*focused_idx, path.clone())));
                    }
                    save_file(path, &comp.content)?;
                    if let Some(comp_mut) = components.get_mut(*focused_idx) {
                        comp_mut.modified = false;
                    }
                    remove_component(components, focused_idx, *focused_idx);
                } else {
                    return Ok(Action::Prompt(PromptAction::ConfirmSaveAs(*focused_idx, String::new())));
                }
            }
        }
        "qa" => {
            // Check for any modified buffers
            for i in 0..components.len() {
                if components[i].component_type == ComponentType::Buffer && components[i].modified {
                    // For now, let's just prompt for the first modified one
                    // or we could have a ConfirmQuitAll
                    return Ok(Action::Prompt(PromptAction::ConfirmQuit(i)));
                }
            }
            components.clear();
            *focused_idx = 0;
            return Ok(Action::Quit);
        }
        "wa" => {
            for i in 0..components.len() {
                if components[i].component_type == ComponentType::Buffer
                    && let Some(path) = components[i].file_path.clone() {
                        save_file(&path, &components[i].content)?;
                        components[i].modified = false;
                }
            }
            push_notification(components, "Saved all buffers".to_string(), config)?;
        }
        "wqa" => {
            for i in 0..components.len() {
                if components[i].component_type == ComponentType::Buffer
                    && let Some(path) = components[i].file_path.clone() {
                        save_file(&path, &components[i].content)?;
                        components[i].modified = false;
                }
            }
            components.clear();
            *focused_idx = 0;
            return Ok(Action::Quit);
        }
        "e" => {
            if parts.len() > 1 {
                let path = parts[1];
                let content_str = open_file(path)?;
                log(format!("Opened file: {}", path))?;
                let mut content: Vec<String> = content_str.lines().map(|s| s.to_string()).collect();
                if content.is_empty() {
                    content.push("".to_string());
                }
                components.push(Component::new(
                    content,
                    ComponentType::Buffer,
                    Some(path.to_string()),
                    config,
                ));
                *focused_idx = components.len() - 1;
            }
        }
        _ => {
            log(format!("Unknown command: {}", parts[0]))?;
        }
    }

    Ok(Action::None)
}
