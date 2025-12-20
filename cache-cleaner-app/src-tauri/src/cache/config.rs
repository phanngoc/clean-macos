use super::custom_scanner::CustomScannerConfig;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub custom_scanners: Vec<CustomScannerConfig>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn add_scanner(&mut self, config: CustomScannerConfig) {
        self.custom_scanners.retain(|s| s.id != config.id);
        self.custom_scanners.push(config);
    }

    pub fn remove_scanner(&mut self, id: &str) -> bool {
        let len = self.custom_scanners.len();
        self.custom_scanners.retain(|s| s.id != id);
        self.custom_scanners.len() < len
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    Ok(home.join(".cache-cleaner/config.json"))
}
