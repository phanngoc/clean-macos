use super::ExtensionInfo;
use crate::utils::filesystem;
use anyhow::Result;
use std::path::PathBuf;

pub fn get_chrome_support_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join("Library/Application Support/Google/Chrome"))
}

pub fn get_cache_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join("Library/Caches/Google/Chrome"))
}

pub fn detect() -> bool {
    get_cache_path().map(|p| p.exists()).unwrap_or(false)
}

/// Scan all Chrome profiles for large extensions (>100MB)
pub fn scan_large_extensions(threshold_mb: u64) -> Result<Vec<ExtensionInfo>> {
    let base = match get_chrome_support_path() {
        Some(p) if p.exists() => p,
        _ => return Ok(vec![]),
    };

    let threshold_bytes = threshold_mb * 1024 * 1024;
    let mut large_extensions = Vec::new();

    // Scan Default and Profile N directories
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let profile_path = entry.path();
            let profile_name = entry.file_name().to_string_lossy().to_string();
            
            // Only check Default and Profile* directories
            if !profile_name.starts_with("Profile") && profile_name != "Default" {
                continue;
            }

            let extensions_dir = profile_path.join("Extensions");
            if !extensions_dir.exists() {
                continue;
            }

            // Scan each extension
            if let Ok(ext_entries) = std::fs::read_dir(&extensions_dir) {
                for ext_entry in ext_entries.flatten() {
                    let ext_path = ext_entry.path();
                    if !ext_path.is_dir() {
                        continue;
                    }

                    let ext_id = ext_entry.file_name().to_string_lossy().to_string();
                    let size = filesystem::calculate_dir_size_sync(&ext_path).unwrap_or(0);

                    if size >= threshold_bytes {
                        large_extensions.push(ExtensionInfo {
                            id: ext_id,
                            path: ext_path,
                            size,
                            profile: profile_name.clone(),
                        });
                    }
                }
            }
        }
    }

    // Sort by size descending
    large_extensions.sort_by(|a, b| b.size.cmp(&a.size));
    Ok(large_extensions)
}
