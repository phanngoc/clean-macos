use crate::cache::{CacheInfo, CacheType, CleanResult};
use crate::cache::paths::MacPaths;
use std::fs;

pub fn get_yarn_cache_info() -> CacheInfo {
    let path = MacPaths::yarn_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::Yarn,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn get_pnpm_cache_info() -> CacheInfo {
    let path = MacPaths::pnpm_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::Pnpm,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn get_pip_cache_info() -> CacheInfo {
    let path = MacPaths::pip_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::Pip,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn get_cocoapods_cache_info() -> CacheInfo {
    let path = MacPaths::cocoapods_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::CocoaPods,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn get_gradle_cache_info() -> CacheInfo {
    let path = MacPaths::gradle_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::Gradle,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn get_cargo_cache_info() -> CacheInfo {
    let path = MacPaths::cargo_cache();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::Cargo,
        path,
        size,
        exists,
        item_count,
    }
}

pub fn clean_package_cache(cache_type: CacheType, dry_run: bool) -> CleanResult {
    let path = match cache_type {
        CacheType::Yarn => MacPaths::yarn_cache(),
        CacheType::Pnpm => MacPaths::pnpm_cache(),
        CacheType::Pip => MacPaths::pip_cache(),
        CacheType::CocoaPods => MacPaths::cocoapods_cache(),
        CacheType::Gradle => MacPaths::gradle_cache(),
        CacheType::Cargo => MacPaths::cargo_cache(),
        _ => return CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: "Invalid package manager cache type".to_string(),
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

fn get_cache_stats(path: &std::path::Path) -> (u64, usize, bool) {
    if path.exists() {
        match calculate_dir_size(path) {
            Ok((s, c)) => (s, c, true),
            Err(_) => (0, 0, false),
        }
    } else {
        (0, 0, false)
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
