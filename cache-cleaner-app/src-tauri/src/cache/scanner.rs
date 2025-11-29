use super::{CacheInfo, CacheType};
use crate::utils::filesystem;
use anyhow::Result;

pub async fn scan_all() -> Result<Vec<CacheInfo>> {
    let mut caches = Vec::new();
    
    for cache_type in [CacheType::Npm, CacheType::Chrome, CacheType::CacheDir] {
        if let Ok(info) = scan_cache(&cache_type).await {
            caches.push(info);
        }
    }
    
    Ok(caches)
}

pub async fn scan_cache(cache_type: &CacheType) -> Result<CacheInfo> {
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

pub async fn get_size(cache_type: &CacheType) -> Result<u64> {
    let path = get_cache_path(cache_type)?;
    if path.exists() {
        filesystem::calculate_dir_size(&path).await
    } else {
        Ok(0)
    }
}

fn get_cache_path(cache_type: &CacheType) -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    
    Ok(match cache_type {
        CacheType::Npm => home.join(".npm"),
        CacheType::Chrome => home.join("Library/Caches/Google/Chrome"),
        CacheType::CacheDir => home.join(".cache"),
        CacheType::ChromeExtensions => home.join("Library/Application Support/Google/Chrome"),
    })
}
