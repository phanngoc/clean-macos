use super::{CacheType, CleanResult};
use crate::cache::{browser_caches, dev_tools, package_managers};
use crate::utils::filesystem;
use anyhow::Result;

pub async fn clean(cache_type: &CacheType, dry_run: bool) -> Result<CleanResult> {
    match cache_type {
        // Browser caches
        CacheType::Safari | CacheType::Firefox | CacheType::Arc => {
            Ok(browser_caches::clean_browser_cache(cache_type.clone(), dry_run))
        }
        
        // Package managers
        CacheType::Yarn | CacheType::Pnpm | CacheType::Pip | 
        CacheType::CocoaPods | CacheType::Gradle | CacheType::Cargo => {
            Ok(package_managers::clean_package_cache(cache_type.clone(), dry_run))
        }
        
        // Development tools
        CacheType::XcodeDerivedData | CacheType::XcodeArchives | CacheType::XcodeSimulators => {
            Ok(dev_tools::clean_xcode_cache(cache_type.clone(), dry_run))
        }
        
        // Existing cache types
        CacheType::Cursor => {
            clean_cursor_cache(dry_run).await
        }
        
        _ => {
            clean_directory_cache(cache_type, dry_run).await
        }
    }
}

async fn clean_cursor_cache(dry_run: bool) -> Result<CleanResult> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let cursor_cache_paths = get_cursor_cache_paths(&home);
    
    let mut total_size = 0u64;
    let mut item_count = 0usize;
    let mut existing_paths = Vec::new();
    
    for path in &cursor_cache_paths {
        if path.exists() {
            total_size += filesystem::calculate_dir_size(path).await?;
            item_count += filesystem::count_items(path)?;
            existing_paths.push(path.clone());
        }
    }
    
    if existing_paths.is_empty() {
        return Ok(CleanResult {
            cache_type: CacheType::Cursor,
            freed_bytes: 0,
            items_removed: 0,
            success: true,
            message: "Cursor cache directories do not exist".to_string(),
            dry_run,
        });
    }
    
    if dry_run {
        return Ok(CleanResult {
            cache_type: CacheType::Cursor,
            freed_bytes: total_size,
            items_removed: item_count,
            success: true,
            message: format!("Would free {} bytes ({} items)", total_size, item_count),
            dry_run: true,
        });
    }
    
    // Clean contents of each cache directory
    for path in &existing_paths {
        filesystem::remove_dir_contents(path)?;
    }
    
    Ok(CleanResult {
        cache_type: CacheType::Cursor,
        freed_bytes: total_size,
        items_removed: item_count,
        success: true,
        message: format!("Freed {} bytes ({} items)", total_size, item_count),
        dry_run: false,
    })
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

async fn clean_directory_cache(cache_type: &CacheType, dry_run: bool) -> Result<CleanResult> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    
    let path = match cache_type {
        CacheType::Npm => home.join(".npm"),
        CacheType::Chrome => home.join("Library/Caches/Google/Chrome"),
        CacheType::CacheDir => home.join(".cache"),
        CacheType::VSCode => home.join("Library/Application Support/Code/WebStorage"),

        _ => {
            return Ok(CleanResult {
                cache_type: cache_type.clone(),
                freed_bytes: 0,
                items_removed: 0,
                success: false,
                message: "Unsupported cache type".to_string(),
                dry_run,
            });
        }
    };
    
    if !path.exists() {
        return Ok(CleanResult {
            cache_type: cache_type.clone(),
            freed_bytes: 0,
            items_removed: 0,
            success: true,
            message: "Cache directory does not exist".to_string(),
            dry_run,
        });
    }
    
    let size_before = filesystem::calculate_dir_size(&path).await?;
    let item_count = filesystem::count_items(&path)?;
    
    if dry_run {
        return Ok(CleanResult {
            cache_type: cache_type.clone(),
            freed_bytes: size_before,
            items_removed: item_count,
            success: true,
            message: format!("Would free {} bytes ({} items)", size_before, item_count),
            dry_run: true,
        });
    }
    
    filesystem::remove_dir_contents(&path)?;
    
    Ok(CleanResult {
        cache_type: cache_type.clone(),
        freed_bytes: size_before,
        items_removed: item_count,
        success: true,
        message: format!("Freed {} bytes", size_before),
        dry_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_unsupported_type() {
        let result = clean(&CacheType::IosBackups, true).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Unsupported cache type");
    }

    #[tokio::test]
    async fn test_clean_dry_run_returns_dry_run_flag() {
        let result = clean(&CacheType::Npm, true).await.unwrap();
        assert!(result.dry_run);
    }

    #[tokio::test]
    async fn test_clean_cursor_nonexistent() {
        // This test verifies behavior when Cursor cache doesn't exist
        let result = clean_cursor_cache(true).await.unwrap();
        // Either files exist or they don't - both are valid
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_clean_result_structure() {
        let result = clean(&CacheType::Chrome, true).await.unwrap();
        // Verify the result has expected structure
        assert!(matches!(result.cache_type, CacheType::Chrome));
        assert!(result.dry_run);
    }

    #[tokio::test]
    async fn test_clean_browser_caches() {
        let result = clean(&CacheType::Safari, true).await.unwrap();
        assert!(matches!(result.cache_type, CacheType::Safari));
    }

    #[tokio::test]
    async fn test_clean_package_managers() {
        let result = clean(&CacheType::Yarn, true).await.unwrap();
        assert!(matches!(result.cache_type, CacheType::Yarn));
    }

    #[tokio::test]
    async fn test_clean_dev_tools() {
        let result = clean(&CacheType::XcodeDerivedData, true).await.unwrap();
        assert!(matches!(result.cache_type, CacheType::XcodeDerivedData));
    }
}
