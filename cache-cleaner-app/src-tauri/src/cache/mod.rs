pub mod scanner;
pub mod cleaner;
pub mod npm;
pub mod chrome;
pub mod cache_dir;
pub mod indexeddb;
pub mod large_caches;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    Npm,
    Chrome,
    CacheDir,
    ChromeExtensions,
    VSCode,
    Cursor,
}

impl CacheType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(CacheType::Npm),
            "chrome" => Ok(CacheType::Chrome),
            "cache_dir" | "cachedir" => Ok(CacheType::CacheDir),
            "chromeextensions" | "chrome_extensions" => Ok(CacheType::ChromeExtensions),
            "vscode" | "code" => Ok(CacheType::VSCode),
            "cursor" => Ok(CacheType::Cursor),
            _ => Err(format!("Unknown cache type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    pub cache_type: CacheType,
    pub path: PathBuf,
    pub size: u64,
    pub exists: bool,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub cache_type: CacheType,
    pub freed_bytes: u64,
    pub items_removed: usize,
    pub success: bool,
    pub message: String,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub id: String,
    pub path: PathBuf,
    pub size: u64,
    pub profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbItem {
    pub profile: String,
    pub origin: String,
    pub path: PathBuf,
    pub size: u64,
    pub over_threshold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedDbCleanResult {
    pub total_freed_bytes: u64,
    pub items_removed: usize,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeCacheEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeCachesCleanResult {
    pub total_freed_bytes: u64,
    pub items_removed: usize,
    pub success: bool,
    pub message: String,
}
