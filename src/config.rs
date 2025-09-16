use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub version: VersionConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VersionConfig {
    pub initial_version: Option<String>,
    pub tag_prefix: Option<String>,
    pub tag_suffix: Option<String>,
    pub files: Option<Vec<FileUpdateConfig>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FileUpdateConfig {
    pub path: String,
    pub marker: String,
    pub template: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: VersionConfig {
                initial_version: Some("0.1.0".to_string()),
                tag_prefix: Some("v".to_string()),
                tag_suffix: None,
                files: Some(vec![FileUpdateConfig {
                    path: "Cargo.toml".to_string(),
                    marker: "0.0.0+local".to_string(),
                    template: None,
                }]),
            },
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        if !path.exists() {
            println!("⚠️  Configuration file not found, using default configuration");
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file {:?}: {}", path, e))?;

        let config = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse TOML config {:?}: {}", path, e))?;

        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let content = toml::to_string(self)
            .map_err(|e| format!("Failed to serialize config to TOML: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write config file {:?}: {}", path, e))?;

        Ok(())
    }
}
