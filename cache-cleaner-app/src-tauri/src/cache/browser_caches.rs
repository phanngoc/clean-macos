use crate::cache::{CacheInfo, CacheType, CleanResult};
use crate::cache::paths::MacPaths;
use std::fs;

pub fn get_safari_cache_info() -> CacheInfo {
    let path = MacPaths::safari_cache();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = if path.exists() {
        match calculate_dir_size(&path) {
            Ok((s, c)) => (s, c, true),
            Err(_) => (0, 0, false),
        }
    } else {
        (0, 0, false)
    };

    CacheInfo {
        cache_type: CacheType::Safari,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn get_firefox_cache_info() -> CacheInfo {
    let path = MacPaths::firefox_profiles();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = if path.exists() {
        match calculate_dir_size(&path) {
            Ok((s, c)) => (s, c, true),
            Err(_) => (0, 0, false),
        }
    } else {
        (0, 0, false)
    };

    CacheInfo {
        cache_type: CacheType::Firefox,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn get_arc_cache_info() -> CacheInfo {
    let path = MacPaths::arc_cache();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = if path.exists() {
        match calculate_dir_size(&path) {
            Ok((s, c)) => (s, c, true),
            Err(_) => (0, 0, false),
        }
    } else {
        (0, 0, false)
    };

    CacheInfo {
        cache_type: CacheType::Arc,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn clean_browser_cache(cache_type: CacheType, dry_run: bool) -> CleanResult {
    let path = match cache_type {
        CacheType::Safari => MacPaths::safari_cache(),
        CacheType::Firefox => MacPaths::firefox_profiles(),
        CacheType::Arc => MacPaths::arc_cache(),
        _ => return CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: "Invalid browser cache type".to_string(),
            dry_run,
        },
    };

    if !path.exists() {
        return CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: true,
            message: "Cache directory does not exist".to_string(),
            dry_run,
        };
    }

    let (size_before, items_before) = match calculate_dir_size(&path) {
        Ok((s, c)) => (s, c),
        Err(e) => return CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: format!("Failed to calculate size: {}", e),
            dry_run,
        },
    };

    if dry_run {
        return CleanResult {
            cache_type,
            freed_bytes: size_before,
            items_removed: items_before,
            success: true,
            message: format!("Would clean {} items ({} bytes)", items_before, size_before),
            dry_run,
        };
    }

    match fs::remove_dir_all(&path) {
        Ok(_) => CleanResult {
            cache_type,
            freed_bytes: size_before,
            items_removed: items_before,
            success: true,
            message: format!("Successfully cleaned {} items", items_before),
            dry_run,
        },
        Err(e) => CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: format!("Failed to clean cache: {}", e),
            dry_run,
        },
    }
}

fn calculate_dir_size(path: &std::path::Path) -> Result<(u64, usize), std::io::Error> {
    let mut total_size = 0;
    let mut item_count = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            item_count += 1;

            if metadata.is_dir() {
                let (sub_size, sub_count) = calculate_dir_size(&entry.path())?;
                total_size += sub_size;
                item_count += sub_count;
            } else {
                total_size += metadata.len();
            }
        }
    }

    Ok((total_size, item_count))
}
