use super::NpmCacheEntry;
use crate::utils::filesystem;
use anyhow::Result;
use std::path::{Path, PathBuf};

// Danh sách các NPM cache paths cần scan
// Có thể dễ dàng thêm paths mới vào đây
const NPM_CACHE_PATHS: &[&str] = &[
    "_cacache/content-v2",
    // Có thể thêm nhiều paths khác sau này:
    // "_cacache/index-v5",
    // "_cacache/tmp",
];

/// Scans ~/.npm for specific cache subdirectories
pub async fn scan_npm_caches() -> Result<Vec<NpmCacheEntry>> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let npm_dir = home.join(".npm");

    if !npm_dir.exists() {
        return Ok(Vec::new());
    }

    let npm_dir = npm_dir.to_path_buf();
    let entries = tokio::task::spawn_blocking(move || {
        scan_npm_caches_sync(&npm_dir)
    })
    .await??;

    Ok(entries)
}

fn scan_npm_caches_sync(npm_dir: &Path) -> Result<Vec<NpmCacheEntry>> {
    let mut npm_entries = Vec::new();

    for relative_path in NPM_CACHE_PATHS {
        let full_path = npm_dir.join(relative_path);
        
        if !full_path.exists() {
            continue;
        }

        // Chỉ xử lý thư mục
        if !full_path.is_dir() {
            continue;
        }

        // Tính toán kích thước thư mục
        let size = match filesystem::calculate_dir_size_sync(&full_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Warning: Could not calculate size for {}: {}", full_path.display(), e);
                continue;
            }
        };

        // Tạo tên hiển thị từ relative_path
        let name = relative_path
            .split('/')
            .last()
            .unwrap_or(relative_path)
            .to_string();
        
        npm_entries.push(NpmCacheEntry {
            name,
            path: full_path.to_string_lossy().to_string(),
            size_bytes: size,
            relative_path: relative_path.to_string(),
        });
    }

    // Sắp xếp theo kích thước giảm dần
    npm_entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    Ok(npm_entries)
}

/// Removes the specified NPM cache directories
pub async fn remove_npm_caches(paths: Vec<String>) -> Result<super::NpmCachesCleanResult> {
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let npm_dir = home.join(".npm");
    let npm_dir_str = npm_dir.to_string_lossy().to_string();

    // Validate tất cả paths đều nằm trong ~/.npm để đảm bảo an toàn
    for path_str in &paths {
        if !path_str.starts_with(&npm_dir_str) {
            return Err(anyhow::anyhow!(
                "Invalid path: {} is not within ~/.npm",
                path_str
            ));
        }
    }

    let paths: Vec<PathBuf> = paths.iter().map(|s| PathBuf::from(s)).collect();
    
    let result = tokio::task::spawn_blocking(move || {
        remove_npm_caches_sync(&paths)
    })
    .await??;

    Ok(result)
}

fn remove_npm_caches_sync(paths: &[PathBuf]) -> Result<super::NpmCachesCleanResult> {
    let mut total_freed = 0u64;
    let mut items_removed = 0usize;
    let mut errors = Vec::new();

    for path in paths {
        if !path.exists() {
            continue;
        }

        // Tính toán kích thước trước khi xóa
        let size = filesystem::calculate_dir_size_sync(path).unwrap_or(0);

        // Xóa thư mục
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
        format!("Successfully removed {} NPM cache directory(ies)", items_removed)
    } else {
        format!(
            "Removed {} NPM cache directory(ies), {} errors: {}",
            items_removed,
            errors.len(),
            errors.join("; ")
        )
    };

    Ok(super::NpmCachesCleanResult {
        total_freed_bytes: total_freed,
        items_removed,
        success: errors.is_empty(),
        message,
    })
}

