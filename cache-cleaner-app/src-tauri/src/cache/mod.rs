pub mod scanner;
pub mod cleaner;
pub mod npm;
pub mod cache_dir;
pub mod indexeddb;
pub mod large_caches;
pub mod npm_caches;
pub mod paths;
pub mod browser_caches;
pub mod dev_tools;
pub mod package_managers;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheType {
    Npm,
    Chrome,
    CacheDir,

    VSCode,
    Cursor,
    Safari,
    Firefox,
    Arc,
    Yarn,
    Pnpm,
    Pip,
    CocoaPods,
    Gradle,
    Cargo,
    XcodeDerivedData,
    XcodeArchives,
    XcodeSimulators,
    SystemCaches,
    UserLogs,
    TempFiles,
    IosBackups,
}

impl CacheType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "npm" => Ok(CacheType::Npm),
            "chrome" => Ok(CacheType::Chrome),
            "cache_dir" | "cachedir" => Ok(CacheType::CacheDir),

            "vscode" | "code" => Ok(CacheType::VSCode),
            "cursor" => Ok(CacheType::Cursor),
            "safari" => Ok(CacheType::Safari),
            "firefox" => Ok(CacheType::Firefox),
            "arc" => Ok(CacheType::Arc),
            "yarn" => Ok(CacheType::Yarn),
            "pnpm" => Ok(CacheType::Pnpm),
            "pip" => Ok(CacheType::Pip),
            "cocoapods" => Ok(CacheType::CocoaPods),
            "gradle" => Ok(CacheType::Gradle),
            "cargo" => Ok(CacheType::Cargo),
            "xcode_derived_data" | "xcodederiveddata" => Ok(CacheType::XcodeDerivedData),
            "xcode_archives" | "xcodearchives" => Ok(CacheType::XcodeArchives),
            "xcode_simulators" | "xcodesimulators" => Ok(CacheType::XcodeSimulators),
            "system_caches" | "systemcaches" => Ok(CacheType::SystemCaches),
            "user_logs" | "userlogs" => Ok(CacheType::UserLogs),
            "temp_files" | "tempfiles" | "tmp" => Ok(CacheType::TempFiles),
            "ios_backups" | "iosbackups" => Ok(CacheType::IosBackups),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmCacheEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmCachesCleanResult {
    pub total_freed_bytes: u64,
    pub items_removed: usize,
    pub success: bool,
    pub message: String,
}
