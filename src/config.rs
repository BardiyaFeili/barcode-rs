use crate::args::Args;
use crate::log::log;
use crate::window::{BorderStyle, HorizontalAnchor, VerticalAnchor};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub status_line: StatusLineConfig,
    pub notification: NotificationConfig,
    pub input: InputConfig,
    #[serde(skip)]
    pub keymap_path: Option<PathBuf>,
    #[serde(skip)]
    pub theme_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct StatusLineConfig {
    pub enabled: bool,
    pub text_left: String,
    pub text_center: String,
    pub text_right: String,
    pub left_end: String,
    pub right_end: String,
}

impl Default for StatusLineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            text_left: "{mode} {file}".to_string(),
            text_center: "{user} in {host} using the great Barcode".to_string(),
            text_right: "{percent} {cursor}".to_string(),
            left_end: "".to_string(),
            right_end: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct NotificationConfig {
    pub enabled: bool,
    pub h_anchor: HorizontalAnchor,
    pub v_anchor: VerticalAnchor,
    pub border_style: BorderStyle,
    pub timeout_secs: u64,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            h_anchor: HorizontalAnchor::Right,
            v_anchor: VerticalAnchor::Top,
            border_style: BorderStyle::Rounded,
            timeout_secs: 3,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct InputConfig {
    pub h_anchor: HorizontalAnchor,
    pub v_anchor: VerticalAnchor,
    pub border_style: BorderStyle,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            h_anchor: HorizontalAnchor::Center,
            v_anchor: VerticalAnchor::Top,
            border_style: BorderStyle::Rounded,
        }
    }
}

pub fn resolve_config_files(args: &Args) -> Result<Config, Box<dyn Error>> {
    let paths = resolve_config_paths(args.config_home.clone())?;
    let mut files: HashMap<&str, PathBuf> = HashMap::new();

    // 1. Check CLI overrides
    if let Some(cfg) = &args.config_file {
        files.insert("config", cfg.clone());
    }
    if let Some(keymap) = &args.keymap_config {
        files.insert("keymap", keymap.clone());
    }
    if let Some(theme) = &args.theme_config {
        files.insert("theme", theme.clone());
    }

    // 2. Search standard paths for missing files
    for path in &paths {
        let targets = [
            ("config", "config.toml"),
            ("keymap", "keymap.toml"),
            ("theme", "theme.toml"),
        ];
        for (key, filename) in targets {
            if !files.contains_key(key) {
                let candidate = path.join(filename);
                if candidate.exists() {
                    files.insert(key, candidate);
                }
            }
        }
        if files.len() == 3 {
            break;
        }
    }

    // 3. Load main config
    let mut config = if let Some(path) = files.get("config") {
        let content = fs::read_to_string(path)?;
        log(format!("Loading config from {:?}", path))?;
        toml::from_str(&content)?
    } else {
        log("No config file found, using defaults")?;
        Config::default()
    };

    // 4. Store paths for keymap and theme
    config.keymap_path = files.get("keymap").cloned();
    config.theme_path = files.get("theme").cloned();

    if let Some(path) = &config.keymap_path {
        log(format!("Keymap file found: {:?}", path))?;
    }
    if let Some(path) = &config.theme_path {
        log(format!("Theme file found: {:?}", path))?;
    }

    Ok(config)
}

fn resolve_config_paths(config_home: Option<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut paths: Vec<PathBuf> = Vec::with_capacity(4);

    if let Some(path) = config_home
        && path.exists()
    {
        paths.push(path);
    }

    if let Ok(val) = env::var("BARCODE_CONFIG_DIR") {
        let path = PathBuf::from(val);
        if path.exists() {
            paths.push(path);
        }
    }

    if let Ok(val) = env::var("XDG_CONFIG_HOME") {
        let path = PathBuf::from(val).join("barcode");
        if path.exists() {
            paths.push(path);
        }
    }

    let path_str = shellexpand::tilde("~/.config/barcode").into_owned();
    let path = PathBuf::from(path_str);
    if path.exists() {
        paths.push(path);
    }

    let mut seen = std::collections::HashSet::new();
    paths.retain(|p| seen.insert(p.clone()));

    Ok(paths)
}
