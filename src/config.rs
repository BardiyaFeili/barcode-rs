use crate::{args::Args, log::log};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::{error::Error, path::PathBuf};
use crate::window::{HorizontalAnchor, VerticalAnchor, BorderStyle};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub status_line: StatusLineConfig,
    #[serde(default)]
    pub notification: NotificationConfig,
    #[serde(default)]
    pub input: InputConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StatusLineConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_text_left")]
    pub text_left: String,
    #[serde(default = "default_text_center")]
    pub text_center: String,
    #[serde(default = "default_text_right")]
    pub text_right: String,
    #[serde(default = "default_left_end")]
    pub left_end: String,
    #[serde(default = "default_right_end")]
    pub right_end: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotificationConfig {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_notif_h_anchor")]
    pub h_anchor: HorizontalAnchor,
    #[serde(default = "default_notif_v_anchor")]
    pub v_anchor: VerticalAnchor,
    #[serde(default = "default_notif_x")]
    pub x: u16,
    #[serde(default = "default_notif_y")]
    pub y: u16,
    #[serde(default = "default_notif_width")]
    pub width: u16,
    #[serde(default = "default_notif_height")]
    pub height: u16,
    #[serde(default = "default_notif_border")]
    pub border_style: BorderStyle,
    #[serde(default = "default_notif_timeout")]
    pub timeout_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InputConfig {
    #[serde(default = "default_input_h_anchor")]
    pub h_anchor: HorizontalAnchor,
    #[serde(default = "default_input_v_anchor")]
    pub v_anchor: VerticalAnchor,
    #[serde(default = "default_input_x")]
    pub x: u16,
    #[serde(default = "default_input_y")]
    pub y: u16,
    #[serde(default = "default_input_width")]
    pub width: u16,
    #[serde(default = "default_input_height")]
    pub height: u16,
    #[serde(default = "default_input_border")]
    pub border_style: BorderStyle,
}

fn default_enabled() -> bool { true }
fn default_text_left() -> String { "{mode} {file}".to_string() }
fn default_text_center() -> String { "".to_string() }
fn default_text_right() -> String { "{time} {date}".to_string() }
fn default_left_end() -> String { "".to_string() }
fn default_right_end() -> String { "".to_string() }

fn default_notif_h_anchor() -> HorizontalAnchor { HorizontalAnchor::Right }
fn default_notif_v_anchor() -> VerticalAnchor { VerticalAnchor::Top }
fn default_notif_x() -> u16 { 2 }
fn default_notif_y() -> u16 { 1 }
fn default_notif_width() -> u16 { 30 }
fn default_notif_height() -> u16 { 3 }
fn default_notif_border() -> BorderStyle { BorderStyle::Rounded }
fn default_notif_timeout() -> u64 { 3 }

fn default_input_h_anchor() -> HorizontalAnchor { HorizontalAnchor::Center }
fn default_input_v_anchor() -> VerticalAnchor { VerticalAnchor::Bottom }
fn default_input_x() -> u16 { 0 }
fn default_input_y() -> u16 { 1 }
fn default_input_width() -> u16 { 60 }
fn default_input_height() -> u16 { 3 }
fn default_input_border() -> BorderStyle { BorderStyle::Rounded }

impl Default for StatusLineConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            text_left: default_text_left(),
            text_center: default_text_center(),
            text_right: default_text_right(),
            left_end: default_left_end(),
            right_end: default_right_end(),
        }
    }
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            h_anchor: default_notif_h_anchor(),
            v_anchor: default_notif_v_anchor(),
            x: default_notif_x(),
            y: default_notif_y(),
            width: default_notif_width(),
            height: default_notif_height(),
            border_style: default_notif_border(),
            timeout_secs: default_notif_timeout(),
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            h_anchor: default_input_h_anchor(),
            v_anchor: default_input_v_anchor(),
            x: default_input_x(),
            y: default_input_y(),
            width: default_input_width(),
            height: default_input_height(),
            border_style: default_input_border(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            status_line: StatusLineConfig::default(),
            notification: NotificationConfig::default(),
            input: InputConfig::default(),
        }
    }
}

pub fn resolve_config_files(args: &Args) -> Result<Config, Box<dyn Error>> {
    let paths = resolve_config_paths(args.config_home.clone())?;

    let mut files: HashMap<&str, PathBuf> = HashMap::new();

    if let Some(cfg) = args.config_file.as_ref() {
        files.insert("config", cfg.clone());
    }
    if let Some(keymap) = args.keymap_config.as_ref() {
        files.insert("keymap", keymap.clone());
    }
    if let Some(theme) = args.theme_config.as_ref() {
        files.insert("theme", theme.clone());
    }

    for path in &paths {
        if !files.contains_key("config") {
            let candidate = path.join("config.toml");
            if candidate.exists() {
                files.insert("config", candidate);
            }
        }
        if !files.contains_key("keymap") {
            let candidate = path.join("keymap.toml");
            if candidate.exists() {
                files.insert("keymap", candidate);
            }
        }
        if !files.contains_key("theme") {
            let candidate = path.join("theme.toml");
            if candidate.exists() {
                files.insert("theme", candidate);
            }
        }
        if files.len() == 3 { break; }
    }

    let config = if let Some(path) = files.get("config") {
        let content = fs::read_to_string(path)?;
        log(format!("Loading config from {:?}", path))?;
        toml::from_str(&content)?
    } else {
        log("No config file found, using defaults")?;
        Config::default()
    };

    Ok(config)
}

fn resolve_config_paths(config_home: Option<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths: Vec<PathBuf> = Vec::new();

    if let Some(path) = config_home {
        if path.exists() { paths.push(path); }
    }

    let env_name = "BARCODE_CONFIG_DIR";
    if let Ok(val) = env::var(env_name) {
        let path = PathBuf::from(val);
        if path.exists() { paths.push(path); }
    }

    let env_name = "XDG_CONFIG_HOME";
    if let Ok(val) = env::var(env_name) {
        let path = PathBuf::from(format!("{}/barcode", val));
        if path.exists() { paths.push(path); }
    }

    let path_str = shellexpand::tilde("~/.config/barcode").into_owned();
    let path = PathBuf::from(path_str);
    if path.exists() { paths.push(path); }

    Ok(paths)
}
