use crate::dns::types::AppConfig;
use std::fs;
use std::path::{Path, PathBuf};
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

pub fn load_config_from_path(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::new());
    }

    let content = fs::read_to_string(path)?;
    let stripped = json_comments::StripComments::new(content.as_bytes());
    let config: AppConfig = serde_json::from_reader(stripped)?;

    Ok(config)
}

pub fn save_config_to_path(config: &AppConfig, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    load_config_from_path(&config_path)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = get_config_path()?;
    save_config_to_path(config, &config_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dns::types::{DnsProfile, DnsServerEntry};
    use tempfile::TempDir;

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
        assert_eq!(config.profiles.len(), 0);
    }

    #[test]
    fn test_load_config_from_path_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.jsonc");

        let result = load_config_from_path(&config_path);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.profiles.len(), 0);
    }

    #[test]
    fn test_load_config_from_path_valid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let mut config = AppConfig::new();
        let profile = DnsProfile::new("Test Profile".to_string());
        config.add_profile(profile);

        save_config_to_path(&config, &config_path).unwrap();

        let loaded = load_config_from_path(&config_path).unwrap();
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "Test Profile");
    }

    #[test]
    fn test_load_config_from_path_jsonc_with_comments() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let jsonc_content = r#"{
  // This is a comment
  "profiles": [
    {
      "id": "test-id",
      "name": "Test Profile", // inline comment
      "settings": {
        "ipv4": {
          "enabled": true,
          "primary": {
            "address": "8.8.8.8",
            "doh_mode": "Off",
            "doh_template": "",
            "allow_fallback": true
          },
          "secondary": {
            "address": "",
            "doh_mode": "Off",
            "doh_template": "",
            "allow_fallback": true
          }
        },
        "ipv6": {
          "enabled": false,
          "primary": {
            "address": "",
            "doh_mode": "Off",
            "doh_template": "",
            "allow_fallback": true
          },
          "secondary": {
            "address": "",
            "doh_mode": "Off",
            "doh_template": "",
            "allow_fallback": true
          }
        }
      }
    }
  ]
}"#;

        fs::write(&config_path, jsonc_content).unwrap();

        let loaded = load_config_from_path(&config_path).unwrap();
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "Test Profile");
        assert_eq!(loaded.profiles[0].id, "test-id");
        assert!(loaded.profiles[0].settings.ipv4.enabled);
        assert_eq!(loaded.profiles[0].settings.ipv4.primary.address, "8.8.8.8");
    }

    #[test]
    fn test_load_config_from_path_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let invalid_json = r#"{ "profiles": [ invalid json ] }"#;
        fs::write(&config_path, invalid_json).unwrap();

        let result = load_config_from_path(&config_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Json(_)));
    }

    #[test]
    fn test_save_config_to_path_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.jsonc");

        let mut config = AppConfig::new();
        let mut profile = DnsProfile::new("Roundtrip Test".to_string());
        profile.settings.ipv4.enabled = true;
        profile.settings.ipv4.primary = DnsServerEntry {
            address: "1.1.1.1".to_string(),
            doh_mode: crate::dns::types::DohMode::Off,
            doh_template: String::new(),
            allow_fallback: true,
        };
        profile.settings.ipv4.secondary = DnsServerEntry {
            address: "1.0.0.1".to_string(),
            doh_mode: crate::dns::types::DohMode::Off,
            doh_template: String::new(),
            allow_fallback: false,
        };
        config.add_profile(profile);

        save_config_to_path(&config, &config_path).unwrap();

        let loaded = load_config_from_path(&config_path).unwrap();
        assert_eq!(loaded, config);
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "Roundtrip Test");
        assert!(loaded.profiles[0].settings.ipv4.enabled);
        assert_eq!(loaded.profiles[0].settings.ipv4.primary.address, "1.1.1.1");
        assert_eq!(
            loaded.profiles[0].settings.ipv4.secondary.address,
            "1.0.0.1"
        );
        assert!(!loaded.profiles[0].settings.ipv4.secondary.allow_fallback);
    }

    #[test]
    fn test_save_config_to_path_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("nested")
            .join("deep")
            .join("config.jsonc");

        assert!(!nested_path.parent().unwrap().exists());

        let mut config = AppConfig::new();
        config.add_profile(DnsProfile::new("Test".to_string()));

        let result = save_config_to_path(&config, &nested_path);
        assert!(result.is_ok());
        assert!(nested_path.exists());
        assert!(nested_path.parent().unwrap().exists());

        let loaded = load_config_from_path(&nested_path).unwrap();
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "Test");
    }
}
