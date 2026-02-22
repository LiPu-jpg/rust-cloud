use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: String,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub sync_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            server: "http://127.0.0.1:3000".to_string(),
            device_id: None,
            device_name: None,
            sync_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("rustcloud"),
        }
    }
}

pub fn load() -> Result<Config, anyhow::Error> {
    let config_path = config_path()?;

    if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save(config: &Config) -> Result<(), anyhow::Error> {
    let config_path = config_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;
    std::fs::write(&config_path, content)?;

    Ok(())
}

fn config_path() -> Result<PathBuf, anyhow::Error> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    Ok(config_dir.join("rustcloud").join("config.toml"))
}
