use std::collections::HashMap;
use std::fmt::format;
use std::{error::Error, path::PathBuf};
use std::env;
use crate::{args::Args, log::log};

pub fn resolve_config_files(args: &Args) -> Result<(), Box<dyn Error>> {
    let paths = resolve_config_paths(args.config_home.clone())?;

    let mut files: HashMap<&str, PathBuf> = HashMap::new();

    // 1️⃣ Check CLI flags first
    if let Some(cfg) = args.config_file.as_ref() {
        files.insert("config", cfg.clone());
    }
    if let Some(keymap) = args.keymap_config.as_ref() {
        files.insert("keymap", keymap.clone());
    }
    if let Some(theme) = args.theme_config.as_ref() {
        files.insert("theme", theme.clone());
    }

    // 2️⃣ If a file wasn't provided via flags, check paths
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

        // If all files found, we can break early
        if files.len() == 3 {
            break;
        }
    }
    
    log(format!("{:?}", files))?;

    Ok(())
}

fn resolve_config_paths(config_home: Option<PathBuf>) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    log("Starting the process of finding paths")?;
    let mut paths: Vec<PathBuf> = Vec::new();
    
    if let Some(path) = config_home {
        if check_path(&path, "config-home") {
            paths.push(path);
        }
    } else {
        log("No argument given using --config-home")?;
    }
    
    let env_name = "BARCODE_CONFIG_DIR";
    if check_env_var(env_name) {
        paths.push(PathBuf::from(env::var(env_name).unwrap()));
    }
    
    let env_name = "XDG_CONFIG_HOME";
    if check_env_var(env_name) {
        let path = PathBuf::from(format!("{}/barcode", env::var(env_name).unwrap()));
        if check_path(&path, "XDG_CONIG_DIR/barcode") {
            paths.push(path);
        }
    }
    
    let path = PathBuf::from("~/.config/barcode");
    if check_path(&path, "~/.config/barcode") {
        paths.push(path);
    }
   
    Ok(paths)
}

fn check_path(path: &PathBuf, name: &str) -> bool {
    if path.exists() {
        log(format!("Path in {} added", name)).unwrap();
        true
    } else {
        log(format!("Path given using {} doesn't exist", name)).unwrap();
        false
    }
}

fn check_env_var(name: &str) -> bool {
    match env::var(name) {
        Ok(path) => {
            let path = PathBuf::from(path);
            if check_path(&path, "env var") {
                true
            } else {
                false
            }
        }
        Err(env::VarError::NotPresent) => {
            log(format!("{} is not set.", name)).unwrap();
            false
        }
        Err(env::VarError::NotUnicode(_)) => {
            log(format!("{} contains invalid Unicode.", name)).unwrap();
            false
        }
    }
}