#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod utils;

use cache::{
    CacheInfo, CacheType, CleanResult, IndexedDbCleanResult, IndexedDbItem,
    LargeCacheEntry, LargeCachesCleanResult, NpmCacheEntry, NpmCachesCleanResult,
    config::AppConfig,
    custom_scanner::CustomScannerConfig,
    registry::ScannerRegistry,
    scanner_trait::{ScanResult, CleanResultGeneric},
    smart_suggestions::{FolderSuggestion, SmartSuggestionsCleanResult},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::OnceCell;

static REGISTRY: OnceCell<Arc<ScannerRegistry>> = OnceCell::const_new();

async fn get_registry() -> &'static Arc<ScannerRegistry> {
    REGISTRY.get_or_init(|| async {
        let registry = Arc::new(ScannerRegistry::new());
        // Load saved custom scanners
        if let Ok(config) = AppConfig::load() {
            for scanner_config in config.custom_scanners {
                let _ = registry.register(scanner_config).await;
            }
        }
        registry
    }).await
}

#[derive(Serialize, Deserialize)]
pub struct PermissionStatus {
    pub full_disk_access: bool,
    pub home_accessible: bool,
}

#[tauri::command]
async fn scan_caches() -> Result<Vec<CacheInfo>, String> {
    cache::scanner::scan_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_cache_size(cache_type: String) -> Result<u64, String> {
    let ct = CacheType::from_str(&cache_type).map_err(|e| e.to_string())?;
    cache::scanner::get_size(&ct).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_cache(cache_type: String, dry_run: bool) -> Result<CleanResult, String> {
    println!(
        "[Rust] clean_cache called: cache_type={}, dry_run={}",
        cache_type, dry_run
    );

    let ct = CacheType::from_str(&cache_type).map_err(|e| {
        eprintln!("[Rust] Failed to parse CacheType from '{}': {}", cache_type, e);
        e.to_string()
    })?;

    let result = cache::cleaner::clean(&ct, dry_run)
        .await
        .map_err(|e| {
            eprintln!(
                "[Rust] cache::cleaner::clean error for {:?}, dry_run={}: {}",
                ct, dry_run, e
            );
            e.to_string()
        })?;

    println!(
        "[Rust] clean_cache finished: type={:?}, freed_bytes={}, items_removed={}, dry_run={}",
        result.cache_type, result.freed_bytes, result.items_removed, result.dry_run
    );

    Ok(result)
}

#[tauri::command]
async fn check_chrome_running() -> Result<bool, String> {
    utils::permissions::is_chrome_running().map_err(|e| e.to_string())
}

#[tauri::command]
async fn check_permissions() -> Result<PermissionStatus, String> {
    Ok(PermissionStatus {
        full_disk_access: utils::permissions::has_full_disk_access(),
        home_accessible: utils::permissions::can_access_home(),
    })
}



#[tauri::command]
async fn scan_indexed_db_items(
    threshold_mb: Option<u64>,
) -> Result<Vec<IndexedDbItem>, String> {
    // Default alert threshold is 10MB if not specified.
    let mb = threshold_mb.unwrap_or(10);
    let threshold_bytes = mb * 1024 * 1024;
    cache::indexeddb::scan_indexed_db(threshold_bytes).map_err(|e| e.to_string())
}

#[tauri::command]
async fn clean_indexed_db_items(
    paths: Vec<String>,
    dry_run: bool,
) -> Result<IndexedDbCleanResult, String> {
    cache::indexeddb::clean_indexed_db_items(paths, dry_run).map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_large_caches() -> Result<Vec<LargeCacheEntry>, String> {
    cache::large_caches::scan_large_caches()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_large_caches(paths: Vec<String>) -> Result<LargeCachesCleanResult, String> {
    cache::large_caches::remove_large_caches(paths)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn scan_npm_caches() -> Result<Vec<NpmCacheEntry>, String> {
    cache::npm_caches::scan_npm_caches()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_npm_caches(paths: Vec<String>) -> Result<NpmCachesCleanResult, String> {
    cache::npm_caches::remove_npm_caches(paths)
        .await
        .map_err(|e| e.to_string())
}

// === Custom Scanner Commands ===

#[tauri::command]
async fn register_custom_scanner(config: CustomScannerConfig) -> Result<(), String> {
    let registry = get_registry().await;
    registry.register(config.clone()).await.map_err(|e| e.to_string())?;
    
    // Persist to config
    let mut app_config = AppConfig::load().unwrap_or_default();
    app_config.add_scanner(config);
    app_config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn list_custom_scanners() -> Result<Vec<CustomScannerConfig>, String> {
    let registry = get_registry().await;
    Ok(registry.list().await)
}

#[tauri::command]
async fn remove_custom_scanner(id: String) -> Result<bool, String> {
    let registry = get_registry().await;
    let removed = registry.unregister(&id).await;
    
    if removed {
        let mut app_config = AppConfig::load().unwrap_or_default();
        app_config.remove_scanner(&id);
        app_config.save().map_err(|e| e.to_string())?;
    }
    Ok(removed)
}

#[tauri::command]
async fn scan_custom_caches() -> Result<Vec<ScanResult>, String> {
    let registry = get_registry().await;
    Ok(registry.scan_all_custom().await)
}

#[tauri::command]
async fn clean_custom_cache(id: String, dry_run: bool) -> Result<CleanResultGeneric, String> {
    let registry = get_registry().await;
    registry.clean_custom(&id, dry_run).await.map_err(|e| e.to_string())
}

// === Smart Suggestions Commands ===

#[tauri::command]
async fn scan_smart_suggestions(min_size_mb: Option<u64>, max_age_days: Option<u64>) -> Result<Vec<FolderSuggestion>, String> {
    cache::smart_suggestions::scan_suggestions(
        min_size_mb.unwrap_or(100),
        max_age_days.unwrap_or(30),
    ).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_folder_suggestion_info(path: String) -> Result<FolderSuggestion, String> {
    cache::smart_suggestions::get_folder_info(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_smart_suggestions(paths: Vec<String>) -> Result<SmartSuggestionsCleanResult, String> {
    cache::smart_suggestions::remove_suggested_folders(paths).await.map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            scan_caches,
            get_cache_size,
            clean_cache,
            check_chrome_running,
            check_permissions,
            scan_indexed_db_items,
            clean_indexed_db_items,
            scan_large_caches,
            remove_large_caches,
            scan_npm_caches,
            remove_npm_caches,
            // Custom scanner commands
            register_custom_scanner,
            list_custom_scanners,
            remove_custom_scanner,
            scan_custom_caches,
            clean_custom_cache,
            // Smart suggestions commands
            scan_smart_suggestions,
            get_folder_suggestion_info,
            remove_smart_suggestions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
