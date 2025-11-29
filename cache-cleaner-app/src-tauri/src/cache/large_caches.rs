use super::LargeCacheEntry;
use crate::utils::filesystem;
use anyhow::Result;
use std::path::{Path, PathBuf};

const ONE_GB: u64 = 1_073_741_824; // 1 GB in bytes

/// Scans ~/Library/Caches for subdirectories larger than 1GB
pub async fn scan_large_caches() -> Result<Vec<LargeCacheEntry>> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let caches_dir = home.join("Library/Caches");

    if !caches_dir.exists() {
        return Ok(Vec::new());
    }

    let caches_dir = caches_dir.to_path_buf();
    let entries = tokio::task::spawn_blocking(move || {
        scan_large_caches_sync(&caches_dir)
    })
    .await??;

    Ok(entries)
}

fn scan_large_caches_sync(caches_dir: &Path) -> Result<Vec<LargeCacheEntry>> {
    let mut large_entries = Vec::new();

    let entries = std::fs::read_dir(caches_dir)?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Only process directories
        if !path.is_dir() {
            continue;
        }

        // Calculate directory size
        let size = match filesystem::calculate_dir_size_sync(&path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: Could not calculate size for {}: {}", path.display(), e);
                continue;
            }
        };

        // Only include directories larger than 1GB
        if size > ONE_GB {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string();
            
            large_entries.push(LargeCacheEntry {
                name,
                path: path.to_string_lossy().to_string(),
                size_bytes: size,
            });
        }
    }

    // Sort by size descending
    large_entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(large_entries)
}

/// Removes the specified cache directories
pub async fn remove_large_caches(paths: Vec<String>) -> Result<super::LargeCachesCleanResult> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let caches_dir = home.join("Library/Caches");
    let caches_dir_str = caches_dir.to_string_lossy().to_string();

    // Validate all paths are within ~/Library/Caches
    for path_str in &paths {
        if !path_str.starts_with(&caches_dir_str) {
            return Err(anyhow::anyhow!(
                "Invalid path: {} is not within ~/Library/Caches",
                path_str
            ));
        }
    }

    let paths: Vec<PathBuf> = paths.iter().map(|s| PathBuf::from(s)).collect();
    
    let result = tokio::task::spawn_blocking(move || {
        remove_large_caches_sync(&paths)
    })
    .await??;

    Ok(result)
}

fn remove_large_caches_sync(paths: &[PathBuf]) -> Result<super::LargeCachesCleanResult> {
    let mut total_freed = 0u64;
    let mut items_removed = 0usize;
    let mut errors = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }

        // Calculate size before deletion
        let size = filesystem::calculate_dir_size_sync(path).unwrap_or(0);

        // Remove the directory
        match std::fs::remove_dir_all(path) {
            Ok(_) => {
                total_freed += size;
                items_removed += 1;
            }
            Err(e) => {
                errors.push(format!("{}: {}", path.display(), e));
            }
        }
    }

    let message = if errors.is_empty() {
        format!("Successfully removed {} directories", items_removed)
    } else {
        format!(
            "Removed {} directories, {} errors: {}",
            items_removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(super::LargeCachesCleanResult {
        total_freed_bytes: total_freed,
        items_removed,
        success: errors.is_empty(),
        message,
    })
}

