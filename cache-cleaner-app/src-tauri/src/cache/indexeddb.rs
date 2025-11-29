use super::{IndexedDbCleanResult, IndexedDbItem};
use crate::utils::filesystem;
use anyhow::Result;
use std::path::PathBuf;

/// Scan Chrome profiles for IndexedDB origins and their sizes.
/// Chỉ trả về các origin có dung lượng >= `threshold_bytes`.
pub fn scan_indexed_db(threshold_bytes: u64) -> Result<Vec<IndexedDbItem>> {
    let base = match super::chrome::get_chrome_support_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(vec![]),
    };

    let mut items = Vec::new();

    // Scan Default and Profile N directories
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let profile_path = entry.path();
            let profile_name = entry.file_name().to_string_lossy().to_string();

            // Only check Default and Profile* directories
            if !profile_name.starts_with("Profile") && profile_name != "Default" {
                continue;
            }

            let indexed_db_dir = profile_path.join("IndexedDB");
            if !indexed_db_dir.exists() {
                continue;
            }

            if let Ok(db_entries) = std::fs::read_dir(&indexed_db_dir) {
                for db_entry in db_entries.flatten() {
                    let db_path = db_entry.path();
                    if !db_path.is_dir() {
                        continue;
                    }

                    let origin = db_entry.file_name().to_string_lossy().to_string();
                    let size = filesystem::calculate_dir_size_sync(&db_path).unwrap_or(0);

                    if size >= threshold_bytes {
                        items.push(IndexedDbItem {
                            profile: profile_name.clone(),
                            origin,
                            path: db_path,
                            size,
                            over_threshold: true,
                        });
                    }
                }
            }
        }
    }

    // Sort by size descending so the largest consumers appear first.
    items.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(items)
}

/// Clean the given IndexedDB folders. Returns how many bytes would / did get freed.
pub fn clean_indexed_db_items(paths: Vec<String>, dry_run: bool) -> Result<IndexedDbCleanResult> {
    let mut total_freed = 0u64;
    let mut items_removed = 0usize;

    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if !path.exists() {
            continue;
        }

        let size = filesystem::calculate_dir_size_sync(&path).unwrap_or(0);
        total_freed += size;
        items_removed += 1;

        if !dry_run {
            // Best-effort removal – ignore individual errors but continue.
            if let Err(e) = std::fs::remove_dir_all(&path) {
                eprintln!(
                    "[Rust] clean_indexed_db_items: failed to remove {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(IndexedDbCleanResult {
        total_freed_bytes: total_freed,
        items_removed,
        dry_run,
    })
}


