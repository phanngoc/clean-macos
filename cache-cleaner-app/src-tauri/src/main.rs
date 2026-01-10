#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cache;
mod utils;

use cache::{
    CacheInfo, CacheType, CleanResult, IndexedDbCleanResult, IndexedDbItem,
    NpmCacheEntry, NpmCachesCleanResult,
    config::AppConfig,
    custom_scanner::CustomScannerConfig,
    registry::ScannerRegistry,
    scanner_trait::{ScanResult, CleanResultGeneric},
    smart_suggestions::{FolderSuggestion, SmartSuggestionsCleanResult},
    docker::{DockerScanResult, DockerCleanResult, DockerSuggestion},
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

// === Docker Cleanup Commands ===

/// Check if Docker is installed and daemon is running
#[tauri::command]
async fn check_docker_status() -> Result<bool, String> {
    Ok(cache::docker::is_docker_running().await)
}

/// Scan all Docker resources (containers, images, volumes, networks)
#[tauri::command]
async fn scan_docker() -> Result<DockerScanResult, String> {
    cache::docker::scan_docker_resources()
        .await
        .map_err(|e| e.to_string())
}

/// Get smart suggestions for Docker cleanup
#[tauri::command]
async fn get_docker_suggestions() -> Result<Vec<DockerSuggestion>, String> {
    cache::docker::get_docker_suggestions()
        .await
        .map_err(|e| e.to_string())
}

/// Remove specific Docker containers
#[tauri::command]
async fn clean_docker_containers(ids: Vec<String>, force: bool) -> Result<DockerCleanResult, String> {
    cache::docker::remove_containers(ids, force)
        .await
        .map_err(|e| e.to_string())
}

/// Remove specific Docker images
#[tauri::command]
async fn clean_docker_images(ids: Vec<String>, force: bool) -> Result<DockerCleanResult, String> {
    cache::docker::remove_images(ids, force)
        .await
        .map_err(|e| e.to_string())
}

/// Remove specific Docker volumes
#[tauri::command]
async fn clean_docker_volumes(names: Vec<String>) -> Result<DockerCleanResult, String> {
    cache::docker::remove_volumes(names)
        .await
        .map_err(|e| e.to_string())
}

/// Remove specific Docker networks
#[tauri::command]
async fn clean_docker_networks(ids: Vec<String>) -> Result<DockerCleanResult, String> {
    cache::docker::remove_networks(ids)
        .await
        .map_err(|e| e.to_string())
}

/// Prune Docker system (all unused resources)
#[tauri::command]
async fn docker_system_prune(all: bool, include_volumes: bool) -> Result<DockerCleanResult, String> {
    cache::docker::docker_system_prune(all, include_volumes)
        .await
        .map_err(|e| e.to_string())
}

/// Prune Docker builder cache
#[tauri::command]
async fn docker_builder_prune() -> Result<DockerCleanResult, String> {
    cache::docker::docker_builder_prune()
        .await
        .map_err(|e| e.to_string())
}

/// Prune stopped containers
#[tauri::command]
async fn docker_prune_containers() -> Result<DockerCleanResult, String> {
    cache::docker::prune_containers()
        .await
        .map_err(|e| e.to_string())
}

/// Prune unused images (dangling or all)
#[tauri::command]
async fn docker_prune_images(all: bool) -> Result<DockerCleanResult, String> {
    cache::docker::prune_images(all)
        .await
        .map_err(|e| e.to_string())
}

/// Prune unused volumes
#[tauri::command]
async fn docker_prune_volumes() -> Result<DockerCleanResult, String> {
    cache::docker::prune_volumes()
        .await
        .map_err(|e| e.to_string())
}

/// Prune unused networks
#[tauri::command]
async fn docker_prune_networks() -> Result<DockerCleanResult, String> {
    cache::docker::prune_networks()
        .await
        .map_err(|e| e.to_string())
}

/// Clean Docker resources based on suggestions
#[tauri::command]
async fn clean_docker_suggestions(suggestions: Vec<DockerSuggestion>) -> Result<DockerCleanResult, String> {
    cache::docker::clean_docker_suggestions(suggestions)
        .await
        .map_err(|e| e.to_string())
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
            // Docker cleanup commands
            check_docker_status,
            scan_docker,
            get_docker_suggestions,
            clean_docker_containers,
            clean_docker_images,
            clean_docker_volumes,
            clean_docker_networks,
            docker_system_prune,
            docker_builder_prune,
            docker_prune_containers,
            docker_prune_images,
            docker_prune_volumes,
            docker_prune_networks,
            clean_docker_suggestions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
