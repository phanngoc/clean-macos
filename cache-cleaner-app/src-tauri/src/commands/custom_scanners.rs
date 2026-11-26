//! User-defined scanners: kept in an in-memory registry and persisted to `AppConfig`.

use super::{CmdResult, IntoCmdResult};
use cache_cleaner::cache::{
    config::AppConfig,
    custom_scanner::CustomScannerConfig,
    registry::ScannerRegistry,
    scanner_trait::{CleanResultGeneric, ScanResult},
};
use tokio::sync::OnceCell;

static REGISTRY: OnceCell<ScannerRegistry> = OnceCell::const_new();

/// Registry shared by all commands, seeded once from the saved config.
async fn registry() -> &'static ScannerRegistry {
    REGISTRY
        .get_or_init(|| async {
            let registry = ScannerRegistry::new();
            if let Ok(config) = AppConfig::load() {
                for scanner_config in config.custom_scanners {
                    let _ = registry.register(scanner_config).await;
                }
            }
            registry
        })
        .await
}

/// Loads the saved config, applies `edit`, and writes it back.
fn update_config(edit: impl FnOnce(&mut AppConfig)) -> CmdResult<()> {
    let mut config = AppConfig::load().unwrap_or_default();
    edit(&mut config);
    config.save().cmd()
}

#[tauri::command]
pub async fn register_custom_scanner(config: CustomScannerConfig) -> CmdResult<()> {
    registry().await.register(config.clone()).await.cmd()?;
    update_config(|c| c.add_scanner(config))
}

#[tauri::command]
pub async fn list_custom_scanners() -> CmdResult<Vec<CustomScannerConfig>> {
    Ok(registry().await.list().await)
}

#[tauri::command]
pub async fn remove_custom_scanner(id: String) -> CmdResult<bool> {
    let removed = registry().await.unregister(&id).await;
    if removed {
        update_config(|c| {
            c.remove_scanner(&id);
        })?;
    }
    Ok(removed)
}

#[tauri::command]
pub async fn scan_custom_caches() -> CmdResult<Vec<ScanResult>> {
    Ok(registry().await.scan_all_custom().await)
}

#[tauri::command]
pub async fn clean_custom_cache(id: String, dry_run: bool) -> CmdResult<CleanResultGeneric> {
    registry().await.clean_custom(&id, dry_run).await.cmd()
}
