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
            let base_path = home.join("Library/Application Support/Cursor/User/globalStorage");
            let file1 = base_path.join("state.vscdb");
            let file2 = base_path.join("state.vscdb.backup");
            
            let mut total_size = 0u64;
            let mut item_count = 0usize;
            let mut exists = false;
            
            if file1.exists() {
                total_size += filesystem::calculate_file_size(&file1).await?;
                item_count += 1;
                exists = true;
            }
            
            if file2.exists() {
                total_size += filesystem::calculate_file_size(&file2).await?;
                item_count += 1;
                exists = true;
            }
            
            Ok(CacheInfo {
                cache_type: cache_type.clone(),
                path: base_path,
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
            // Calculate size of 2 Cursor files
            let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
            let base_path = home.join("Library/Application Support/Cursor/User/globalStorage");
            let file1 = base_path.join("state.vscdb");
            let file2 = base_path.join("state.vscdb.backup");
            
            let mut total_size = 0u64;
            if file1.exists() {
                total_size += filesystem::calculate_file_size(&file1).await?;
            }
            if file2.exists() {
                total_size += filesystem::calculate_file_size(&file2).await?;
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
