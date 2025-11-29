use super::{CacheInfo, CacheType};
use crate::utils::filesystem;
use anyhow::Result;

pub async fn scan_all() -> Result<Vec<CacheInfo>> {
    let mut caches = Vec::new();
    
    for cache_type in [
        CacheType::Npm,
        CacheType::Chrome,
        CacheType::CacheDir,
        CacheType::VSCode,
        CacheType::Cursor,
    ] {
        if let Ok(info) = scan_cache(&cache_type).await {
            caches.push(info);
        }
    }
    
    Ok(caches)
}

pub async fn scan_cache(cache_type: &CacheType) -> Result<CacheInfo> {
    match cache_type {
        CacheType::Cursor => {
            // Cursor has 2 specific files to scan
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
        CacheType::VSCode => {
            // VSCode has a folder to scan
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
        _ => {
            // Other cache types (Npm, Chrome, CacheDir) - scan as directory
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
        CacheType::ChromeExtensions => home.join("Library/Application Support/Google/Chrome"),
        CacheType::VSCode => home.join("Library/Application Support/Code/WebStorage"),
        CacheType::Cursor => home.join("Library/Application Support/Cursor/User/globalStorage"),
    })
}
