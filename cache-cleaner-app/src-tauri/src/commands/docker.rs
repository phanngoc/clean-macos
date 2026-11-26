//! Docker cleanup: status, scan, suggestions, targeted removal and prune.

use super::{CmdResult, IntoCmdResult};
use cache_cleaner::cache::docker::{
    self, DockerCleanResult, DockerScanResult, DockerSuggestion,
};

// --- Status & scan ---

/// Whether Docker is installed and its daemon is reachable.
#[tauri::command]
pub async fn check_docker_status() -> CmdResult<bool> {
    Ok(docker::is_docker_running().await)
}

/// All Docker resources (containers, images, volumes, networks).
#[tauri::command]
pub async fn scan_docker() -> CmdResult<DockerScanResult> {
    docker::scan_docker_resources().await.cmd()
}

#[tauri::command]
pub async fn get_docker_suggestions() -> CmdResult<Vec<DockerSuggestion>> {
    docker::get_docker_suggestions().await.cmd()
}

// --- Remove selected resources ---

#[tauri::command]
pub async fn clean_docker_containers(ids: Vec<String>, force: bool) -> CmdResult<DockerCleanResult> {
    docker::remove_containers(ids, force).await.cmd()
}

#[tauri::command]
pub async fn clean_docker_images(ids: Vec<String>, force: bool) -> CmdResult<DockerCleanResult> {
    docker::remove_images(ids, force).await.cmd()
}

#[tauri::command]
pub async fn clean_docker_volumes(names: Vec<String>) -> CmdResult<DockerCleanResult> {
    docker::remove_volumes(names).await.cmd()
}

#[tauri::command]
pub async fn clean_docker_networks(ids: Vec<String>) -> CmdResult<DockerCleanResult> {
    docker::remove_networks(ids).await.cmd()
}

#[tauri::command]
pub async fn clean_docker_suggestions(
    suggestions: Vec<DockerSuggestion>,
) -> CmdResult<DockerCleanResult> {
    docker::clean_docker_suggestions(suggestions).await.cmd()
}

// --- Prune unused resources ---

#[tauri::command]
pub async fn docker_system_prune(all: bool, include_volumes: bool) -> CmdResult<DockerCleanResult> {
    docker::docker_system_prune(all, include_volumes).await.cmd()
}

#[tauri::command]
pub async fn docker_builder_prune() -> CmdResult<DockerCleanResult> {
    docker::docker_builder_prune().await.cmd()
}

#[tauri::command]
pub async fn docker_prune_containers() -> CmdResult<DockerCleanResult> {
    docker::prune_containers().await.cmd()
}

/// Dangling images only, or every unused image when `all` is set.
#[tauri::command]
pub async fn docker_prune_images(all: bool) -> CmdResult<DockerCleanResult> {
    docker::prune_images(all).await.cmd()
}

#[tauri::command]
pub async fn docker_prune_volumes() -> CmdResult<DockerCleanResult> {
    docker::prune_volumes().await.cmd()
}

#[tauri::command]
pub async fn docker_prune_networks() -> CmdResult<DockerCleanResult> {
    docker::prune_networks().await.cmd()
}
