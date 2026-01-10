use crate::cache::{CacheInfo, CacheType, CleanResult};
use crate::cache::paths::MacPaths;
use std::fs;

pub fn get_xcode_derived_data_info() -> CacheInfo {
    let path = MacPaths::xcode_derived_data();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::XcodeDerivedData,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn get_xcode_archives_info() -> CacheInfo {
    let path = MacPaths::xcode_archives();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::XcodeArchives,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn get_xcode_simulators_info() -> CacheInfo {
    let path = MacPaths::xcode_simulators();
    let path_str = path.display().to_string();
    let (size, item_count, exists) = get_cache_stats(&path);
    
    CacheInfo {
        cache_type: CacheType::XcodeSimulators,
        path: path_str,
        size,
        exists,
        item_count,
    }
}

pub fn clean_xcode_cache(cache_type: CacheType, dry_run: bool) -> CleanResult {
    let path = match cache_type {
        CacheType::XcodeDerivedData => MacPaths::xcode_derived_data(),
        CacheType::XcodeArchives => MacPaths::xcode_archives(),
        CacheType::XcodeSimulators => MacPaths::xcode_simulators(),
        _ => return CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: "Invalid Xcode cache type".to_string(),
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

    // For Xcode caches, we might want to be more careful
    match cache_type {
        CacheType::XcodeDerivedData => clean_derived_data(&path, size_before, items_before, dry_run),
        CacheType::XcodeArchives => clean_archives(&path, size_before, items_before, dry_run),
        CacheType::XcodeSimulators => clean_simulators(&path, size_before, items_before, dry_run),
        _ => CleanResult {
            cache_type,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: "Invalid cache type".to_string(),
            dry_run,
        },
    }
}

fn clean_derived_data(path: &std::path::Path, size_before: u64, items_before: usize, dry_run: bool) -> CleanResult {
    match fs::remove_dir_all(path) {
        Ok(_) => CleanResult {
            cache_type: CacheType::XcodeDerivedData,
            freed_bytes: size_before,
            items_removed: items_before,
            success: true,
            message: "Successfully cleaned Xcode DerivedData".to_string(),
            dry_run,
        },
        Err(e) => CleanResult {
            cache_type: CacheType::XcodeDerivedData,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: format!("Failed to clean DerivedData: {}", e),
            dry_run,
        },
    }
}

fn clean_archives(path: &std::path::Path, size_before: u64, items_before: usize, dry_run: bool) -> CleanResult {
    match fs::remove_dir_all(path) {
        Ok(_) => CleanResult {
            cache_type: CacheType::XcodeArchives,
            freed_bytes: size_before,
            items_removed: items_before,
            success: true,
            message: "Successfully cleaned Xcode Archives".to_string(),
            dry_run,
        },
        Err(e) => CleanResult {
            cache_type: CacheType::XcodeArchives,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: format!("Failed to clean Archives: {}", e),
            dry_run,
        },
    }
}

fn clean_simulators(path: &std::path::Path, size_before: u64, items_before: usize, dry_run: bool) -> CleanResult {
    match fs::remove_dir_all(path) {
        Ok(_) => CleanResult {
            cache_type: CacheType::XcodeSimulators,
            freed_bytes: size_before,
            items_removed: items_before,
            success: true,
            message: "Successfully cleaned Xcode Simulators".to_string(),
            dry_run,
        },
        Err(e) => CleanResult {
            cache_type: CacheType::XcodeSimulators,
            freed_bytes: 0,
            items_removed: 0,
            success: false,
            message: format!("Failed to clean Simulators: {}", e),
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
