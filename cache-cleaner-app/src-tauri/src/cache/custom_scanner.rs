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
            let stats = path_stats(self.resolved_path.clone()).await?;
            (stats.bytes, stats.items)
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

        if dry_run {
            let stats = path_stats(self.resolved_path.clone()).await?;
            return Ok(CleanResultGeneric {
                id: self.config.id.clone(),
                freed_bytes: stats.bytes,
                items_removed: stats.items,
                success: true,
                message: format!("Would free {} bytes", stats.bytes),
                dry_run,
            });
        }

        let path = self.resolved_path.clone();
        let stats = tokio::task::spawn_blocking(move || filesystem::remove_contents(&path, None)).await?;
        let message = if stats.failed == 0 {
            "Cleaned successfully".to_string()
        } else {
            format!("Cleaned with {} items skipped (in use or access denied)", stats.failed)
        };

        Ok(CleanResultGeneric {
            id: self.config.id.clone(),
            freed_bytes: stats.freed_bytes,
            items_removed: stats.removed,
            success: stats.failed == 0 || stats.removed > 0,
            message,
            dry_run,
        })
    }
}

async fn path_stats(path: PathBuf) -> Result<filesystem::PathStats> {
    Ok(tokio::task::spawn_blocking(move || filesystem::path_stats(&path, None)).await?)
}

fn expand_path(path: &str) -> Result<PathBuf> {
    if let Some(rest) = path.strip_prefix("~/").or_else(|| path.strip_prefix("~\\")) {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
        Ok(home.join(rest))
    } else {
        Ok(PathBuf::from(path))
    }
}
