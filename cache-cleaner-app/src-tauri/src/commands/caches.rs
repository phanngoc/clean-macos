//! Built-in cache catalog (scan / size / clean), IndexedDB origins, npm caches.

use super::{CmdResult, IntoCmdResult};
use cache_cleaner::cache::{
    self, CacheInfo, CacheType, CleanResult, IndexedDbCleanResult, IndexedDbItem,
    NpmCacheEntry, NpmCachesCleanResult,
};

const MB: u64 = 1024 * 1024;
/// IndexedDB origins below this size are not reported unless the UI asks otherwise.
const DEFAULT_INDEXED_DB_THRESHOLD_MB: u64 = 10;

fn parse_cache_type(cache_type: &str) -> CmdResult<CacheType> {
    CacheType::from_str(cache_type).map_err(|e| {
        eprintln!("[Rust] Failed to parse CacheType from '{cache_type}': {e}");
        e
    })
}

#[tauri::command]
pub async fn scan_caches() -> CmdResult<Vec<CacheInfo>> {
    cache::scanner::scan_all().await.cmd()
}

#[tauri::command]
pub async fn get_cache_size(cache_type: String) -> CmdResult<u64> {
    let ct = parse_cache_type(&cache_type)?;
    cache::scanner::get_size(&ct).await.cmd()
}

#[tauri::command]
pub async fn clean_cache(cache_type: String, dry_run: bool) -> CmdResult<CleanResult> {
    println!("[Rust] clean_cache called: cache_type={cache_type}, dry_run={dry_run}");
    let ct = parse_cache_type(&cache_type)?;

    let result = cache::cleaner::clean(&ct, dry_run).await.map_err(|e| {
        eprintln!("[Rust] clean failed for {ct:?}, dry_run={dry_run}: {e}");
        e.to_string()
    })?;

    println!(
        "[Rust] clean_cache finished: type={:?}, freed_bytes={}, items_removed={}, dry_run={}",
        result.cache_type, result.freed_bytes, result.items_removed, result.dry_run
    );
    Ok(result)
}

#[tauri::command]
pub async fn scan_indexed_db_items(threshold_mb: Option<u64>) -> CmdResult<Vec<IndexedDbItem>> {
    let threshold_bytes = threshold_mb.unwrap_or(DEFAULT_INDEXED_DB_THRESHOLD_MB) * MB;
    cache::indexeddb::scan_indexed_db(threshold_bytes).cmd()
}

#[tauri::command]
pub async fn clean_indexed_db_items(
    paths: Vec<String>,
    dry_run: bool,
) -> CmdResult<IndexedDbCleanResult> {
    cache::indexeddb::clean_indexed_db_items(paths, dry_run).cmd()
}

#[tauri::command]
pub async fn scan_npm_caches() -> CmdResult<Vec<NpmCacheEntry>> {
    cache::npm_caches::scan_npm_caches().await.cmd()
}

#[tauri::command]
pub async fn remove_npm_caches(paths: Vec<String>) -> CmdResult<NpmCachesCleanResult> {
    cache::npm_caches::remove_npm_caches(paths).await.cmd()
}
