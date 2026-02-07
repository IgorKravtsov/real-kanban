use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("Could not determine config directory")?
        .join("real-kanban");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn global_config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_global_config() -> Result<GlobalConfig> {
    let path = global_config_path()?;
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: GlobalConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let path = global_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(&path, content)?;
    Ok(())
}
