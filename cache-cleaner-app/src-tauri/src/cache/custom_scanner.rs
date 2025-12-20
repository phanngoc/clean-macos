use super::scanner_trait::{CacheCleaner, CacheScanner, CleanResultGeneric, ScanResult};
use crate::utils::filesystem;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScannerConfig {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub min_size_mb: Option<u64>,
}

pub struct CustomScanner {
    config: CustomScannerConfig,
    resolved_path: PathBuf,
}

impl CustomScanner {
    pub fn new(config: CustomScannerConfig) -> Result<Self> {
        let resolved_path = expand_path(&config.path)?;
        Ok(Self { config, resolved_path })
    }

    pub fn config(&self) -> &CustomScannerConfig {
        &self.config
    }
}

#[async_trait]
impl CacheScanner for CustomScanner {
    fn id(&self) -> &str {
        &self.config.id
    }

    fn display_name(&self) -> &str {
        &self.config.name
    }

    async fn scan(&self) -> Result<ScanResult> {
        let exists = self.resolved_path.exists();
        let (size_bytes, item_count) = if exists {
            let size = filesystem::calculate_dir_size(&self.resolved_path).await?;
            let count = filesystem::count_items(&self.resolved_path).unwrap_or(0);
            (size, count)
        } else {
            (0, 0)
        };

        Ok(ScanResult {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            path: self.resolved_path.clone(),
            size_bytes,
            item_count,
            exists,
        })
    }
}

#[async_trait]
impl CacheCleaner for CustomScanner {
    fn id(&self) -> &str {
        &self.config.id
    }

    async fn clean(&self, dry_run: bool) -> Result<CleanResultGeneric> {
        if !self.resolved_path.exists() {
            return Ok(CleanResultGeneric {
                id: self.config.id.clone(),
                freed_bytes: 0,
                items_removed: 0,
                success: true,
                message: "Path does not exist".to_string(),
                dry_run,
            });
        }

        let size = filesystem::calculate_dir_size(&self.resolved_path).await?;
        let count = filesystem::count_items(&self.resolved_path).unwrap_or(0);

        if dry_run {
            return Ok(CleanResultGeneric {
                id: self.config.id.clone(),
                freed_bytes: size,
                items_removed: count,
                success: true,
                message: format!("Would free {} bytes", size),
                dry_run,
            });
        }

        filesystem::remove_dir_contents(&self.resolved_path)?;

        Ok(CleanResultGeneric {
            id: self.config.id.clone(),
            freed_bytes: size,
            items_removed: count,
            success: true,
            message: "Cleaned successfully".to_string(),
            dry_run,
        })
    }
}

fn expand_path(path: &str) -> Result<PathBuf> {
    if path.starts_with("~/") {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(&path[2..]))
    } else {
        Ok(PathBuf::from(path))
    }
}
