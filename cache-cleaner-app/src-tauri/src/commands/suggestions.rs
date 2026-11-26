//! Smart suggestions: large folders that have not been used for a while.

use super::{CmdResult, IntoCmdResult};
use cache_cleaner::cache::smart_suggestions::{
    self, FolderSuggestion, SmartSuggestionsCleanResult,
};

const DEFAULT_MIN_SIZE_MB: u64 = 100;
const DEFAULT_MAX_AGE_DAYS: u64 = 30;

#[tauri::command]
pub async fn scan_smart_suggestions(
    min_size_mb: Option<u64>,
    max_age_days: Option<u64>,
) -> CmdResult<Vec<FolderSuggestion>> {
    smart_suggestions::scan_suggestions(
        min_size_mb.unwrap_or(DEFAULT_MIN_SIZE_MB),
        max_age_days.unwrap_or(DEFAULT_MAX_AGE_DAYS),
    )
    .await
    .cmd()
}

#[tauri::command]
pub async fn get_folder_suggestion_info(path: String) -> CmdResult<FolderSuggestion> {
    smart_suggestions::get_folder_info(&path).await.cmd()
}

#[tauri::command]
pub async fn remove_smart_suggestions(paths: Vec<String>) -> CmdResult<SmartSuggestionsCleanResult> {
    smart_suggestions::remove_suggested_folders(paths).await.cmd()
}
