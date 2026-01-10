pub mod scanner;
pub mod cleaner;
pub mod npm;
pub mod cache_dir;
pub mod indexeddb;
pub mod npm_caches;
pub mod paths;
pub mod browser_caches;
pub mod dev_tools;
pub mod package_managers;

// New modules for custom scanner & smart suggestions
pub mod scanner_trait;
pub mod config;
pub mod custom_scanner;
pub mod registry;
pub mod smart_suggestions;
pub mod parallel_scanner;

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
    pub path: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_type_from_str_basic() {
        assert!(matches!(CacheType::from_str("npm"), Ok(CacheType::Npm)));
        assert!(matches!(CacheType::from_str("chrome"), Ok(CacheType::Chrome)));
        assert!(matches!(CacheType::from_str("cachedir"), Ok(CacheType::CacheDir)));
        assert!(matches!(CacheType::from_str("cache_dir"), Ok(CacheType::CacheDir)));
    }

    #[test]
    fn test_cache_type_from_str_editors() {
        assert!(matches!(CacheType::from_str("vscode"), Ok(CacheType::VSCode)));
        assert!(matches!(CacheType::from_str("code"), Ok(CacheType::VSCode)));
        assert!(matches!(CacheType::from_str("cursor"), Ok(CacheType::Cursor)));
    }

    #[test]
    fn test_cache_type_from_str_browsers() {
        assert!(matches!(CacheType::from_str("safari"), Ok(CacheType::Safari)));
        assert!(matches!(CacheType::from_str("firefox"), Ok(CacheType::Firefox)));
        assert!(matches!(CacheType::from_str("arc"), Ok(CacheType::Arc)));
    }

    #[test]
    fn test_cache_type_from_str_package_managers() {
        assert!(matches!(CacheType::from_str("yarn"), Ok(CacheType::Yarn)));
        assert!(matches!(CacheType::from_str("pnpm"), Ok(CacheType::Pnpm)));
        assert!(matches!(CacheType::from_str("pip"), Ok(CacheType::Pip)));
        assert!(matches!(CacheType::from_str("cocoapods"), Ok(CacheType::CocoaPods)));
        assert!(matches!(CacheType::from_str("gradle"), Ok(CacheType::Gradle)));
        assert!(matches!(CacheType::from_str("cargo"), Ok(CacheType::Cargo)));
    }

    #[test]
    fn test_cache_type_from_str_xcode() {
        assert!(matches!(CacheType::from_str("xcodederiveddata"), Ok(CacheType::XcodeDerivedData)));
        assert!(matches!(CacheType::from_str("xcode_derived_data"), Ok(CacheType::XcodeDerivedData)));
        assert!(matches!(CacheType::from_str("xcodearchives"), Ok(CacheType::XcodeArchives)));
        assert!(matches!(CacheType::from_str("xcodesimulators"), Ok(CacheType::XcodeSimulators)));
    }

    #[test]
    fn test_cache_type_from_str_system() {
        assert!(matches!(CacheType::from_str("systemcaches"), Ok(CacheType::SystemCaches)));
        assert!(matches!(CacheType::from_str("userlogs"), Ok(CacheType::UserLogs)));
        assert!(matches!(CacheType::from_str("tempfiles"), Ok(CacheType::TempFiles)));
        assert!(matches!(CacheType::from_str("tmp"), Ok(CacheType::TempFiles)));
        assert!(matches!(CacheType::from_str("iosbackups"), Ok(CacheType::IosBackups)));
    }

    #[test]
    fn test_cache_type_from_str_case_insensitive() {
        assert!(matches!(CacheType::from_str("NPM"), Ok(CacheType::Npm)));
        assert!(matches!(CacheType::from_str("Chrome"), Ok(CacheType::Chrome)));
        assert!(matches!(CacheType::from_str("VSCODE"), Ok(CacheType::VSCode)));
    }

    #[test]
    fn test_cache_type_from_str_unknown() {
        assert!(CacheType::from_str("unknown").is_err());
        assert!(CacheType::from_str("").is_err());
        assert!(CacheType::from_str("invalid_type").is_err());
    }

    #[test]
    fn test_cache_info_creation() {
        let info = CacheInfo {
            cache_type: CacheType::Npm,
            path: "/test/path".to_string(),
            size: 1024,
            exists: true,
            item_count: 10,
        };
        assert!(info.exists);
        assert_eq!(info.size, 1024);
        assert_eq!(info.item_count, 10);
    }

    #[test]
    fn test_clean_result_creation() {
        let result = CleanResult {
            cache_type: CacheType::Chrome,
            freed_bytes: 2048,
            items_removed: 5,
            success: true,
            message: "Cleaned successfully".to_string(),
            dry_run: false,
        };
        assert!(result.success);
        assert_eq!(result.freed_bytes, 2048);
        assert!(!result.dry_run);
    }

    #[test]
    fn test_indexed_db_item_creation() {
        let item = IndexedDbItem {
            profile: "Default".to_string(),
            origin: "https://example.com".to_string(),
            path: std::path::PathBuf::from("/test/indexeddb"),
            size: 5000,
            over_threshold: true,
        };
        assert!(item.over_threshold);
        assert_eq!(item.profile, "Default");
    }

    #[test]
    fn test_cache_type_serialization() {
        let cache_type = CacheType::Npm;
        let serialized = serde_json::to_string(&cache_type).unwrap();
        assert_eq!(serialized, "\"Npm\"");
        
        let deserialized: CacheType = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, CacheType::Npm));
    }

    #[test]
    fn test_cache_info_serialization() {
        let info = CacheInfo {
            cache_type: CacheType::Chrome,
            path: "/test".to_string(),
            size: 100,
            exists: true,
            item_count: 5,
        };
        let serialized = serde_json::to_string(&info).unwrap();
        assert!(serialized.contains("Chrome"));
        assert!(serialized.contains("100"));
    }
}
