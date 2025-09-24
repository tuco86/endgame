use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG: &str = include_str!("config.yaml");

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub whitelist: Vec<String>,
    pub blacklist: Vec<String>,
}

pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let local_appdata =
        std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA environment variable not found")?;

    let config_dir = PathBuf::from(local_appdata).join("Endgame");

    // Create directory if it doesn't exist
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)?;
        log::info!("Created config directory: {}", config_dir.display());
    }

    Ok(config_dir)
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = config_file_path()?;

    Ok(if config_path.exists() {
        // Load existing config
        let config_content = fs::read_to_string(&config_path)?;
        let config: Config = serde_yaml::from_str(&config_content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        log::info!("Loaded config from: {}", config_path.display());
        config
    } else {
        // Parse the embedded config to return it
        let default_config: Config = serde_yaml::from_str(CONFIG)?;
        log::info!("Loaded default config");
        default_config
    })
}

pub fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(get_config_dir()?.join("config.yaml"))
}

pub fn ensure_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = config_file_path()?;
    if !path.exists() {
        // Write embedded default config
        std::fs::write(&path, CONFIG)?;
        log::info!("Created default config file: {}", path.display());
    }
    Ok(path)
}
