use std::error::Error;
use std::time::Duration;
use crate::{
    component::{Component, ComponentType},
    config::Config,
    file::{open_file, save_file},
    log::log,
};

pub fn handle_command(
    cmd: &str,
    components: &mut Vec<Component>,
    focused_idx: &mut usize,
    config: &Config,
) -> Result<(), Box<dyn Error>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    match parts[0] {
        "w" => {
            if let Some(comp) = components.get(*focused_idx) {
                if let Some(path) = &comp.file_path {
                    save_file(path, &comp.content)?;
                    log(format!("File saved: {}", path))?;
                    push_notification(components, format!("Saved {}", path), config)?;
                } else {
                    push_notification(components, "No file path associated with buffer".to_string(), config)?;
                }
            }
        }
        "q" => {
            if !components.is_empty() {
                components.remove(*focused_idx);
                if *focused_idx >= components.len() && !components.is_empty() {
                    *focused_idx = components.len() - 1;
                }
            }
        }
        "wq" => {
            if let Some(comp) = components.get(*focused_idx)
                && let Some(path) = &comp.file_path {
                    save_file(path, &comp.content)?;
                    log(format!("File saved: {}", path))?;
                    push_notification(components, format!("Saved {}", path), config)?;
            }
            if !components.is_empty() {
                components.remove(*focused_idx);
                if *focused_idx >= components.len() && !components.is_empty() {
                    *focused_idx = components.len() - 1;
                }
            }
        }
        "qa" => {
            components.clear();
            *focused_idx = 0;
        }
        "wa" => {
            for comp in components.iter() {
                if comp.component_type == ComponentType::Buffer
                    && let Some(path) = &comp.file_path {
                        save_file(path, &comp.content)?;
                        log(format!("File saved: {}", path))?;
                }
            }
            push_notification(components, "Saved all buffers".to_string(), config)?;
        }
        "wqa" => {
            for comp in components.iter() {
                if comp.component_type == ComponentType::Buffer
                    && let Some(path) = &comp.file_path {
                        save_file(path, &comp.content)?;
                        log(format!("File saved: {}", path))?;
                }
            }
            components.clear();
            *focused_idx = 0;
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
                ));
                *focused_idx = components.len() - 1;
            }
        }
        _ => {
            log(format!("Unknown command: {}", parts[0]))?;
        }
    }

    Ok(())
}

fn push_notification(components: &mut Vec<Component>, message: String, config: &Config) -> Result<(), Box<dyn Error>> {
    let cfg = &config.notification;
    if !cfg.enabled {
        return Ok(());
    }

    let mut notify = Component::new(
        vec![message],
        ComponentType::Notification,
        None
    ).with_timer(Duration::from_secs(cfg.timeout_secs));
    
    notify.window.window_type = crate::window::WindowType::Floating;
    notify.window.h_anchor = cfg.h_anchor;
    notify.window.v_anchor = cfg.v_anchor;
    notify.window.x = 2;
    notify.window.y = 1;
    notify.window.window_width = 30;
    notify.window.window_height = 3;
    notify.window.border_style = cfg.border_style;
    
    components.push(notify);
    Ok(())
}
