use crate::utils::{access_tracker, filesystem};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSuggestion {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub score: f64,
    pub reasons: Vec<String>,
    pub last_accessed_days_ago: Option<u64>,
}

const SCAN_LOCATIONS: &[&str] = &[
    "~/Library/Caches",
    "~/.cache",
    "~/Library/Logs",
    "~/Downloads",
];

pub async fn scan_suggestions(min_size_mb: u64, max_age_days: u64) -> Result<Vec<FolderSuggestion>> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let min_size_bytes = min_size_mb * 1024 * 1024;
    let mut suggestions = Vec::new();

    for loc in SCAN_LOCATIONS {
        let path = if loc.starts_with("~/") {
            home.join(&loc[2..])
        } else {
            PathBuf::from(loc)
        };

        if !path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if !entry_path.is_dir() {
                    continue;
                }

                if let Ok(suggestion) = analyze_folder(&entry_path, min_size_bytes, max_age_days).await {
                    if let Some(s) = suggestion {
                        suggestions.push(s);
                    }
                }
            }
        }
    }

    suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(suggestions)
}

async fn analyze_folder(path: &PathBuf, min_size_bytes: u64, max_age_days: u64) -> Result<Option<FolderSuggestion>> {
    let size = filesystem::calculate_dir_size(path).await?;
    if size < min_size_bytes {
        return Ok(None);
    }

    let access_info = access_tracker::get_access_info(path)?;
    let days_ago = access_tracker::days_since_access(&access_info);

    let mut score = 0.0;
    let mut reasons = Vec::new();

    // Size score (40%)
    let size_gb = size as f64 / (1024.0 * 1024.0 * 1024.0);
    score += (size_gb.min(10.0) / 10.0) * 40.0;
    if size_gb >= 1.0 {
        reasons.push(format!("{:.1} GB", size_gb));
    } else {
        reasons.push(format!("{:.0} MB", size as f64 / (1024.0 * 1024.0)));
    }

    // Age score (30%)
    if let Some(days) = days_ago {
        if days >= max_age_days {
            score += 30.0;
            reasons.push(format!("Not accessed in {} days", days));
        } else {
            score += (days as f64 / max_age_days as f64) * 30.0;
        }
    }

    // Location score (20%)
    let path_str = path.to_string_lossy();
    if path_str.contains("Caches") || path_str.contains(".cache") {
        score += 20.0;
        reasons.push("Cache folder".to_string());
    } else if path_str.contains("Logs") {
        score += 15.0;
        reasons.push("Log folder".to_string());
    }

    // Item count score (10%)
    let item_count = filesystem::count_items(path).unwrap_or(0);
    if item_count > 1000 {
        score += 10.0;
        reasons.push(format!("{} items", item_count));
    }

    let name = path.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    Ok(Some(FolderSuggestion {
        path: path.to_string_lossy().to_string(),
        name,
        size_bytes: size,
        score,
        reasons,
        last_accessed_days_ago: days_ago,
    }))
}

pub async fn get_folder_info(path: &str) -> Result<FolderSuggestion> {
    let path_buf = PathBuf::from(path);
    analyze_folder(&path_buf, 0, 30).await?
        .ok_or_else(|| anyhow::anyhow!("Could not analyze folder"))
}
