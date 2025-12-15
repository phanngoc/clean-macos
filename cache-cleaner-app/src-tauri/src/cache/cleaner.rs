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
    let base_path = home.join("Library/Application Support/Cursor/User/globalStorage");
    let file1 = base_path.join("state.vscdb");
    let file2 = base_path.join("state.vscdb.backup");
    
    let mut total_size = 0u64;
    let mut item_count = 0usize;
    
    if file1.exists() {
        total_size += filesystem::calculate_file_size(&file1).await?;
        item_count += 1;
    }
    
    if file2.exists() {
        total_size += filesystem::calculate_file_size(&file2).await?;
        item_count += 1;
    }
    
    if item_count == 0 {
        return Ok(CleanResult {
            cache_type: CacheType::Cursor,
            freed_bytes: 0,
            items_removed: 0,
            success: true,
            message: "Cursor cache files do not exist".to_string(),
            dry_run,
        });
    }
    
    if dry_run {
        return Ok(CleanResult {
            cache_type: CacheType::Cursor,
            freed_bytes: total_size,
            items_removed: item_count,
            success: true,
            message: format!("Would free {} bytes ({} files)", total_size, item_count),
            dry_run: true,
        });
    }
    
    // Actually delete the files
    if file1.exists() {
        filesystem::remove_file(&file1)?;
    }
    if file2.exists() {
        filesystem::remove_file(&file2)?;
    }
    
    Ok(CleanResult {
        cache_type: CacheType::Cursor,
        freed_bytes: total_size,
        items_removed: item_count,
        success: true,
        message: format!("Freed {} bytes ({} files)", total_size, item_count),
        dry_run: false,
    })
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
