//! Tauri command layer of the desktop app (GUI binary only).
//!
//! Each submodule groups the commands of one feature area and is a thin
//! adapter over `cache_cleaner::{cache, utils}`: parse arguments, call the
//! core, turn errors into strings the webview can display. Business logic
//! belongs in the library so the CLI can share it.

mod caches;
mod custom_scanners;
mod docker;
mod suggestions;
mod system;

use std::fmt::Display;
use tauri::{ipc::Invoke, Wry};

/// Result type returned to the frontend: errors are serialized as plain strings.
pub type CmdResult<T> = Result<T, String>;

/// Converts any `Result<T, E: Display>` into a [`CmdResult`].
pub trait IntoCmdResult<T> {
    fn cmd(self) -> CmdResult<T>;
}

impl<T, E: Display> IntoCmdResult<T> for Result<T, E> {
    fn cmd(self) -> CmdResult<T> {
        self.map_err(|e| e.to_string())
    }
}

/// Every command exposed to the webview. Command names are the bare function
/// names (e.g. `scan_caches`), so the module split is invisible to the UI.
pub fn handler() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // Built-in cache catalog, IndexedDB, npm
        caches::scan_caches,
        caches::get_cache_size,
        caches::clean_cache,
        caches::scan_indexed_db_items,
        caches::clean_indexed_db_items,
        caches::scan_npm_caches,
        caches::remove_npm_caches,
        // Environment checks
        system::check_chrome_running,
        system::check_permissions,
        // User-defined scanners
        custom_scanners::register_custom_scanner,
        custom_scanners::list_custom_scanners,
        custom_scanners::remove_custom_scanner,
        custom_scanners::scan_custom_caches,
        custom_scanners::clean_custom_cache,
        // Smart suggestions
        suggestions::scan_smart_suggestions,
        suggestions::get_folder_suggestion_info,
        suggestions::remove_smart_suggestions,
        // Docker
        docker::check_docker_status,
        docker::scan_docker,
        docker::get_docker_suggestions,
        docker::clean_docker_containers,
        docker::clean_docker_images,
        docker::clean_docker_volumes,
        docker::clean_docker_networks,
        docker::clean_docker_suggestions,
        docker::docker_system_prune,
        docker::docker_builder_prune,
        docker::docker_prune_containers,
        docker::docker_prune_images,
        docker::docker_prune_volumes,
        docker::docker_prune_networks,
    ]
}
