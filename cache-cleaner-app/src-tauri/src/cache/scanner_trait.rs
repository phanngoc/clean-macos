use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub size_bytes: u64,
    pub item_count: usize,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResultGeneric {
    pub id: String,
    pub freed_bytes: u64,
    pub items_removed: usize,
    pub success: bool,
    pub message: String,
    pub dry_run: bool,
}

#[async_trait]
pub trait CacheScanner: Send + Sync {
    fn id(&self) -> &str;
    fn display_name(&self) -> &str;
    async fn scan(&self) -> Result<ScanResult>;
}

#[async_trait]
pub trait CacheCleaner: Send + Sync {
    fn id(&self) -> &str;
    async fn clean(&self, dry_run: bool) -> Result<CleanResultGeneric>;
}
