use crate::dns::types::AppConfig;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config directory not found")]
    ConfigDirNotFound,
}

pub type Result<T> = std::result::Result<T, ConfigError>;

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .or_else(dirs::data_local_dir)
        .ok_or(ConfigError::ConfigDirNotFound)?;

    let app_config_dir = config_dir.join("windns");
    Ok(app_config_dir.join("config.jsonc"))
}

pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_path = get_config_path()?;
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(config_path)
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(AppConfig::new());
    }

    let content = fs::read_to_string(&config_path)?;
    let stripped = json_comments::StripComments::new(content.as_bytes());
    let config: AppConfig = serde_json::from_reader(stripped)?;

    Ok(config)
}

/// Saves the configuration to the config file.
/// Note: Comments in the original file will not be preserved.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = ensure_config_dir()?;
    let json = serde_json::to_string_pretty(config)?;
    fs::write(&config_path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_path() {
        let path = get_config_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("windns"));
        assert!(path.to_string_lossy().ends_with("config.jsonc"));
    }

    #[test]
    fn test_load_nonexistent_config() {
        let config = AppConfig::new();
        assert_eq!(config.interfaces.len(), 0);
    }
}
