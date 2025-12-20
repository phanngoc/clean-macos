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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartSuggestionsCleanResult {
    pub total_freed_bytes: u64,
    pub items_removed: usize,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
enum LocationType {
    Cache,
    Log,
    Dev,
    AppSupport,
    Unknown,
}

struct FolderFeatures {
    size_mb: u64,
    last_accessed_days: Option<u64>,
    location_type: LocationType,
}

const WHITELIST_PATHS: &[&str] = &[
    "~/Library/Caches",
    "~/Library/Logs",
    "~/Library/Application Support/*/Cache",
    "~/Library/Containers/*/Data/Library/Caches",
    "~/.npm",
    "~/.yarn",
    "~/.cache",
    "~/Library/Developer/Xcode/DerivedData",
];

fn expand_home(path: &str, home: &PathBuf) -> PathBuf {
    if path.starts_with("~/") {
        home.join(&path[2..])
    } else {
        PathBuf::from(path)
    }
}

fn expand_wildcard_paths(home: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    for pattern in WHITELIST_PATHS {
        let expanded = expand_home(pattern, home);
        let path_str = expanded.to_string_lossy();
        
        if path_str.contains('*') {
            // Split at wildcard and expand
            let parts: Vec<&str> = path_str.split('*').collect();
            if parts.len() == 2 {
                let base = PathBuf::from(parts[0].trim_end_matches('/'));
                let suffix = parts[1].trim_start_matches('/');
                
                if base.exists() {
                    if let Ok(entries) = std::fs::read_dir(&base) {
                        for entry in entries.flatten() {
                            if entry.path().is_dir() {
                                let full_path = if suffix.is_empty() {
                                    entry.path()
                                } else {
                                    entry.path().join(suffix)
                                };
                                if full_path.exists() && full_path.is_dir() {
                                    paths.push(full_path);
                                }
                            }
                        }
                    }
                }
            }
        } else if expanded.exists() {
            paths.push(expanded);
        }
    }
    
    paths
}

fn determine_location_type(path: &PathBuf) -> LocationType {
    let path_str = path.to_string_lossy().to_lowercase();
    
    if path_str.contains("deriveddata") || path_str.contains("xcode") {
        LocationType::Dev
    } else if path_str.contains("/logs") {
        LocationType::Log
    } else if path_str.contains("caches") || path_str.contains(".cache") 
        || path_str.contains(".npm") || path_str.contains(".yarn") {
        LocationType::Cache
    } else if path_str.contains("application support") {
        LocationType::AppSupport
    } else {
        LocationType::Unknown
    }
}

fn calculate_size_score(size_mb: u64) -> f64 {
    if size_mb >= 5120 { 1.0 }
    else if size_mb >= 1024 { 0.7 }
    else if size_mb >= 500 { 0.4 }
    else { (size_mb as f64 / 500.0) * 0.4 }
}

fn calculate_age_score(days: Option<u64>) -> f64 {
    match days {
        Some(d) if d >= 180 => 1.0,
        Some(d) if d >= 90 => 0.6,
        Some(d) if d >= 30 => 0.3,
        Some(d) => (d as f64 / 30.0) * 0.3,
        None => 0.0,
    }
}

fn calculate_location_score(location_type: &LocationType) -> f64 {
    match location_type {
        LocationType::Cache | LocationType::Log | LocationType::Dev => 1.0,
        LocationType::AppSupport => 0.6,
        LocationType::Unknown => 0.2,
    }
}

fn generate_reasons(features: &FolderFeatures) -> Vec<String> {
    let mut reasons = Vec::new();
    
    if features.size_mb >= 1024 {
        reasons.push(format!("Large size: {:.1} GB", features.size_mb as f64 / 1024.0));
    } else {
        reasons.push(format!("Size: {} MB", features.size_mb));
    }
    
    if let Some(days) = features.last_accessed_days {
        if days >= 30 {
            reasons.push(format!("Not accessed for {} days", days));
        }
    }
    
    match features.location_type {
        LocationType::Cache => reasons.push("Cache directory".to_string()),
        LocationType::Log => reasons.push("Log directory".to_string()),
        LocationType::Dev => reasons.push("Development cache".to_string()),
        _ => {}
    }
    
    reasons
}

pub async fn scan_suggestions(min_size_mb: u64, max_age_days: u64) -> Result<Vec<FolderSuggestion>> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;
    let min_size_bytes = min_size_mb * 1024 * 1024;
    let mut suggestions = Vec::new();

    let scan_paths = expand_wildcard_paths(&home);

    for scan_path in scan_paths {
        if !scan_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&scan_path) {
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

async fn analyze_folder(path: &PathBuf, min_size_bytes: u64, _max_age_days: u64) -> Result<Option<FolderSuggestion>> {
    let size = filesystem::calculate_dir_size(path).await?;
    if size < min_size_bytes {
        return Ok(None);
    }

    let access_info = access_tracker::get_access_info(path)?;
    let days_ago = access_tracker::days_since_access(&access_info);
    let location_type = determine_location_type(path);
    let size_mb = size / (1024 * 1024);

    let features = FolderFeatures {
        size_mb,
        last_accessed_days: days_ago,
        location_type: location_type.clone(),
    };

    // Score: 40% size + 40% age + 20% location
    let score = calculate_size_score(size_mb) * 0.4
        + calculate_age_score(days_ago) * 0.4
        + calculate_location_score(&location_type) * 0.2;

    let reasons = generate_reasons(&features);

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

pub async fn remove_suggested_folders(paths: Vec<String>) -> Result<SmartSuggestionsCleanResult> {
    let mut total_freed = 0u64;
    let mut items_removed = 0usize;
    let mut errors = Vec::new();

    for path_str in &paths {
        let path = PathBuf::from(path_str);
        if !path.exists() {
            errors.push(format!("Path not found: {}", path_str));
            continue;
        }

        match filesystem::calculate_dir_size(&path).await {
            Ok(size) => {
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    errors.push(format!("Failed to remove {}: {}", path_str, e));
                } else {
                    total_freed += size;
                    items_removed += 1;
                }
            }
            Err(e) => {
                errors.push(format!("Failed to calculate size for {}: {}", path_str, e));
            }
        }
    }

    let success = errors.is_empty();
    let message = if success {
        format!("Successfully removed {} directories", items_removed)
    } else {
        format!("Removed {} directories with {} errors: {}", items_removed, errors.len(), errors.join("; "))
    };

    Ok(SmartSuggestionsCleanResult {
        total_freed_bytes: total_freed,
        items_removed,
        success,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn create_test_file(dir: &std::path::Path, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    fn create_large_test_file(dir: &std::path::Path, name: &str, size_mb: u64) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        let chunk = vec![0u8; 1024 * 1024];
        for _ in 0..size_mb {
            file.write_all(&chunk).unwrap();
        }
        path
    }

    #[test]
    fn test_folder_suggestion_creation() {
        let suggestion = FolderSuggestion {
            path: "/test/path".to_string(),
            name: "test_folder".to_string(),
            size_bytes: 1024 * 1024 * 500,
            score: 0.75,
            reasons: vec!["Large size: 500.0 MB".to_string()],
            last_accessed_days_ago: Some(90),
        };
        assert_eq!(suggestion.path, "/test/path");
        assert_eq!(suggestion.score, 0.75);
    }

    #[test]
    fn test_folder_suggestion_serialization() {
        let suggestion = FolderSuggestion {
            path: "/test/path".to_string(),
            name: "test_folder".to_string(),
            size_bytes: 1024 * 1024 * 100,
            score: 0.5,
            reasons: vec!["Cache folder".to_string()],
            last_accessed_days_ago: None,
        };
        let json = serde_json::to_string(&suggestion).unwrap();
        let deserialized: FolderSuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(suggestion.path, deserialized.path);
    }

    #[test]
    fn test_location_type_cache() {
        assert_eq!(determine_location_type(&PathBuf::from("/Library/Caches/app")), LocationType::Cache);
        assert_eq!(determine_location_type(&PathBuf::from("~/.cache/test")), LocationType::Cache);
        assert_eq!(determine_location_type(&PathBuf::from("~/.npm")), LocationType::Cache);
    }

    #[test]
    fn test_location_type_log() {
        assert_eq!(determine_location_type(&PathBuf::from("/Library/Logs/app")), LocationType::Log);
    }

    #[test]
    fn test_location_type_dev() {
        assert_eq!(determine_location_type(&PathBuf::from("/Library/Developer/Xcode/DerivedData")), LocationType::Dev);
    }

    #[test]
    fn test_size_score_thresholds() {
        assert_eq!(calculate_size_score(5120), 1.0);
        assert_eq!(calculate_size_score(6000), 1.0);
        assert_eq!(calculate_size_score(1024), 0.7);
        assert_eq!(calculate_size_score(500), 0.4);
        assert!(calculate_size_score(250) < 0.4);
    }

    #[test]
    fn test_age_score_thresholds() {
        assert_eq!(calculate_age_score(Some(180)), 1.0);
        assert_eq!(calculate_age_score(Some(200)), 1.0);
        assert_eq!(calculate_age_score(Some(90)), 0.6);
        assert_eq!(calculate_age_score(Some(30)), 0.3);
        assert!(calculate_age_score(Some(15)) < 0.3);
        assert_eq!(calculate_age_score(None), 0.0);
    }

    #[test]
    fn test_location_score() {
        assert_eq!(calculate_location_score(&LocationType::Cache), 1.0);
        assert_eq!(calculate_location_score(&LocationType::Log), 1.0);
        assert_eq!(calculate_location_score(&LocationType::Dev), 1.0);
        assert_eq!(calculate_location_score(&LocationType::AppSupport), 0.6);
        assert_eq!(calculate_location_score(&LocationType::Unknown), 0.2);
    }

    #[test]
    fn test_generate_reasons_large_size() {
        let features = FolderFeatures {
            size_mb: 2048,
            last_accessed_days: None,
            location_type: LocationType::Unknown,
        };
        let reasons = generate_reasons(&features);
        assert!(reasons.iter().any(|r| r.contains("GB")));
    }

    #[test]
    fn test_generate_reasons_small_size() {
        let features = FolderFeatures {
            size_mb: 500,
            last_accessed_days: None,
            location_type: LocationType::Unknown,
        };
        let reasons = generate_reasons(&features);
        assert!(reasons.iter().any(|r| r.contains("MB")));
    }

    #[test]
    fn test_generate_reasons_age() {
        let features = FolderFeatures {
            size_mb: 100,
            last_accessed_days: Some(90),
            location_type: LocationType::Unknown,
        };
        let reasons = generate_reasons(&features);
        assert!(reasons.iter().any(|r| r.contains("Not accessed")));
    }

    #[test]
    fn test_generate_reasons_location() {
        let features = FolderFeatures {
            size_mb: 100,
            last_accessed_days: None,
            location_type: LocationType::Cache,
        };
        let reasons = generate_reasons(&features);
        assert!(reasons.iter().any(|r| r.contains("Cache")));
    }

    #[tokio::test]
    async fn test_analyze_folder_below_min_size() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "small.txt", b"small");
        let result = analyze_folder(&dir.path().to_path_buf(), 100 * 1024 * 1024, 30).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_analyze_folder_meets_min_size() {
        let dir = create_test_dir();
        create_large_test_file(dir.path(), "large.txt", 150);
        let result = analyze_folder(&dir.path().to_path_buf(), 100 * 1024 * 1024, 30).await.unwrap();
        assert!(result.is_some());
        let suggestion = result.unwrap();
        assert!(suggestion.score >= 0.0 && suggestion.score <= 1.0);
    }

    #[tokio::test]
    async fn test_analyze_folder_name_extraction() {
        let dir = create_test_dir();
        let subdir = dir.path().join("test_folder_name");
        fs::create_dir(&subdir).unwrap();
        create_large_test_file(&subdir, "file.txt", 200);
        let result = analyze_folder(&subdir, 0, 30).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test_folder_name");
    }

    #[tokio::test]
    async fn test_scan_suggestions_sorted() {
        let result = scan_suggestions(0, 30).await;
        assert!(result.is_ok());
        let suggestions = result.unwrap();
        for i in 1..suggestions.len() {
            assert!(suggestions[i-1].score >= suggestions[i].score);
        }
    }

    #[tokio::test]
    async fn test_get_folder_info_valid() {
        let dir = create_test_dir();
        create_large_test_file(dir.path(), "test.txt", 100);
        let result = get_folder_info(dir.path().to_str().unwrap()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_folder_info_nonexistent() {
        let result = get_folder_info("/nonexistent/path").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_suggested_folders_empty() {
        let result = remove_suggested_folders(vec![]).await.unwrap();
        assert_eq!(result.items_removed, 0);
        assert_eq!(result.total_freed_bytes, 0);
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_remove_suggested_folders_single() {
        let dir = create_test_dir();
        let subdir = dir.path().join("to_remove");
        fs::create_dir(&subdir).unwrap();
        create_large_test_file(&subdir, "test.txt", 10);
        
        let path_str = subdir.to_string_lossy().to_string();
        let result = remove_suggested_folders(vec![path_str]).await.unwrap();
        
        assert_eq!(result.items_removed, 1);
        assert!(result.total_freed_bytes > 0);
        assert!(result.success);
        assert!(!subdir.exists());
    }

    #[tokio::test]
    async fn test_remove_suggested_folders_multiple() {
        let dir = create_test_dir();
        let subdir1 = dir.path().join("folder1");
        let subdir2 = dir.path().join("folder2");
        fs::create_dir_all(&subdir1).unwrap();
        fs::create_dir_all(&subdir2).unwrap();
        create_large_test_file(&subdir1, "file1.txt", 5);
        create_large_test_file(&subdir2, "file2.txt", 7);
        
        let paths = vec![
            subdir1.to_string_lossy().to_string(),
            subdir2.to_string_lossy().to_string(),
        ];
        let result = remove_suggested_folders(paths).await.unwrap();
        
        assert_eq!(result.items_removed, 2);
        assert!(result.success);
        assert!(!subdir1.exists());
        assert!(!subdir2.exists());
    }

    #[tokio::test]
    async fn test_remove_suggested_folders_nonexistent() {
        let result = remove_suggested_folders(vec!["/nonexistent/path".to_string()]).await.unwrap();
        assert_eq!(result.items_removed, 0);
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_score_normalized_range() {
        let dir = create_test_dir();
        create_large_test_file(dir.path(), "test.txt", 200);
        let result = analyze_folder(&dir.path().to_path_buf(), 0, 30).await.unwrap();
        assert!(result.is_some());
        let suggestion = result.unwrap();
        assert!(suggestion.score >= 0.0 && suggestion.score <= 1.0);
    }

    #[test]
    fn test_smart_suggestions_clean_result_serialization() {
        let result = SmartSuggestionsCleanResult {
            total_freed_bytes: 1000,
            items_removed: 1,
            success: true,
            message: "Test".to_string(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SmartSuggestionsCleanResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.total_freed_bytes, deserialized.total_freed_bytes);
    }
}
