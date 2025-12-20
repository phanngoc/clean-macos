use super::{CacheInfo, CacheType};
use crate::cache::{browser_caches, dev_tools, package_managers, paths::MacPaths};
use crate::utils::filesystem;
use anyhow::Result;

pub async fn scan_all() -> Result<Vec<CacheInfo>> {
    let mut caches = Vec::new();
    
    let cache_types = [
        CacheType::Npm,
        CacheType::Chrome,
        CacheType::CacheDir,
        CacheType::VSCode,
        CacheType::Cursor,
        CacheType::Safari,
        CacheType::Firefox,
        CacheType::Arc,
        CacheType::Yarn,
        CacheType::Pnpm,
        CacheType::Pip,
        CacheType::CocoaPods,
        CacheType::Gradle,
        CacheType::Cargo,
        CacheType::XcodeDerivedData,
        CacheType::XcodeArchives,
        CacheType::XcodeSimulators,
    ];
    
    for cache_type in cache_types {
        if let Ok(info) = scan_cache(&cache_type).await {
            if info.exists && info.size > 0 {
                caches.push(info);
            }
        }
    }
    
    Ok(caches)
}

pub async fn scan_cache(cache_type: &CacheType) -> Result<CacheInfo> {
    match cache_type {
        // Browser caches
        CacheType::Safari => Ok(browser_caches::get_safari_cache_info()),
        CacheType::Firefox => Ok(browser_caches::get_firefox_cache_info()),
        CacheType::Arc => Ok(browser_caches::get_arc_cache_info()),
        
        // Package managers
        CacheType::Yarn => Ok(package_managers::get_yarn_cache_info()),
        CacheType::Pnpm => Ok(package_managers::get_pnpm_cache_info()),
        CacheType::Pip => Ok(package_managers::get_pip_cache_info()),
        CacheType::CocoaPods => Ok(package_managers::get_cocoapods_cache_info()),
        CacheType::Gradle => Ok(package_managers::get_gradle_cache_info()),
        CacheType::Cargo => Ok(package_managers::get_cargo_cache_info()),
        
        // Development tools
        CacheType::XcodeDerivedData => Ok(dev_tools::get_xcode_derived_data_info()),
        CacheType::XcodeArchives => Ok(dev_tools::get_xcode_archives_info()),
        CacheType::XcodeSimulators => Ok(dev_tools::get_xcode_simulators_info()),
        
        // Existing cache types
        CacheType::Cursor => {
            let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
            let cursor_cache_paths = get_cursor_cache_paths(&home);
            
            let mut total_size = 0u64;
            let mut item_count = 0usize;
            let mut exists = false;
            let mut display_path = home.join("Library/Caches/Cursor");
            
            for path in &cursor_cache_paths {
                if path.exists() {
                    exists = true;
                    display_path = path.clone();
                    total_size += filesystem::calculate_dir_size(path).await?;
                    item_count += filesystem::count_items(path)?;
                }
            }
            
            Ok(CacheInfo {
                cache_type: cache_type.clone(),
                path: display_path,
                size: total_size,
                exists,
                item_count,
            })
        }
        _ => {
            let path = get_cache_path(cache_type)?;
            let exists = path.exists();
            let (size, item_count) = if exists {
                let size = filesystem::calculate_dir_size(&path).await?;
                let count = filesystem::count_items(&path)?;
                (size, count)
            } else {
                (0, 0)
            };

            Ok(CacheInfo {
                cache_type: cache_type.clone(),
                path,
                size,
                exists,
                item_count,
            })
        }
    }
}

pub async fn get_size(cache_type: &CacheType) -> Result<u64> {
    match cache_type {
        CacheType::Cursor => {
            let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
            let cursor_cache_paths = get_cursor_cache_paths(&home);
            
            let mut total_size = 0u64;
            for path in cursor_cache_paths {
                if path.exists() {
                    total_size += filesystem::calculate_dir_size(&path).await?;
                }
            }
            Ok(total_size)
        }
        _ => {
            let path = get_cache_path(cache_type)?;
            if path.exists() {
                if path.is_file() {
                    filesystem::calculate_file_size(&path).await
                } else {
                    filesystem::calculate_dir_size(&path).await
                }
            } else {
                Ok(0)
            }
        }
    }
}

/// Get all safe Cursor cache directories
fn get_cursor_cache_paths(home: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    
    // Main cache in Library/Caches (todesktop bundle ID)
    let caches_dir = home.join("Library/Caches");
    if let Ok(entries) = std::fs::read_dir(&caches_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("com.todesktop.") && entry.path().is_dir() {
                paths.push(entry.path());
            }
        }
    }
    
    // Secondary cache directory
    let cursor_cache = caches_dir.join("Cursor");
    if cursor_cache.exists() {
        paths.push(cursor_cache);
    }
    
    // Safe directories in Application Support
    let app_support = home.join("Library/Application Support/Cursor");
    for subdir in ["CachedExtensions", "CachedExtensionVSIXs", "logs"] {
        let path = app_support.join(subdir);
        if path.exists() {
            paths.push(path);
        }
    }
    
    paths
}

fn get_cache_path(cache_type: &CacheType) -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    
    Ok(match cache_type {
        CacheType::Npm => home.join(".npm"),
        CacheType::Chrome => home.join("Library/Caches/Google/Chrome"),
        CacheType::CacheDir => home.join(".cache"),

        CacheType::VSCode => home.join("Library/Application Support/Code/WebStorage"),
        CacheType::Cursor => home.join("Library/Application Support/Cursor/User/globalStorage"),
        CacheType::Safari => MacPaths::safari_cache(),
        CacheType::Firefox => MacPaths::firefox_profiles(),
        CacheType::Arc => MacPaths::arc_cache(),
        CacheType::Yarn => MacPaths::yarn_cache(),
        CacheType::Pnpm => MacPaths::pnpm_cache(),
        CacheType::Pip => MacPaths::pip_cache(),
        CacheType::CocoaPods => MacPaths::cocoapods_cache(),
        CacheType::Gradle => MacPaths::gradle_cache(),
        CacheType::Cargo => MacPaths::cargo_cache(),
        CacheType::XcodeDerivedData => MacPaths::xcode_derived_data(),
        CacheType::XcodeArchives => MacPaths::xcode_archives(),
        CacheType::XcodeSimulators => MacPaths::xcode_simulators(),
        CacheType::SystemCaches => MacPaths::system_caches(),
        CacheType::UserLogs => MacPaths::user_logs(),
        CacheType::TempFiles => MacPaths::tmp(),
        CacheType::IosBackups => MacPaths::ios_backups(),
    })
}
