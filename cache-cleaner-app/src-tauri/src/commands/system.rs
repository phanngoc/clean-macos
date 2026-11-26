//! Environment checks shown as warnings in the UI.

use super::{CmdResult, IntoCmdResult};
use cache_cleaner::utils::permissions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PermissionStatus {
    pub full_disk_access: bool,
    pub home_accessible: bool,
}

#[tauri::command]
pub async fn check_chrome_running() -> CmdResult<bool> {
    permissions::is_chrome_running().cmd()
}

#[tauri::command]
pub async fn check_permissions() -> CmdResult<PermissionStatus> {
    Ok(PermissionStatus {
        full_disk_access: permissions::has_full_disk_access(),
        home_accessible: permissions::can_access_home(),
    })
}
