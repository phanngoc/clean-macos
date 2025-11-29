use super::{CacheType, CleanResult};
use crate::utils::filesystem;
use anyhow::Result;

pub async fn clean(cache_type: &CacheType, dry_run: bool) -> Result<CleanResult> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    
    let path = match cache_type {
        CacheType::Npm => home.join(".npm"),
        CacheType::Chrome => home.join("Library/Caches/Google/Chrome"),
        CacheType::CacheDir => home.join(".cache"),
        CacheType::ChromeExtensions => {
            return Ok(CleanResult {
                cache_type: cache_type.clone(),
                freed_bytes: 0,
                items_removed: 0,
                success: false,
                message: "Use Chrome to manage extensions".to_string(),
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
