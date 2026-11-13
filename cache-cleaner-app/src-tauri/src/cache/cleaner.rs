use super::{CacheType, CleanResult};
use crate::cache::paths;
use crate::cache::scanner::{existing_locations, total_stats};
use crate::utils::filesystem::{self, RemovalStats};
use anyhow::Result;

/// Empties every location of `cache_type`. Locked / access-denied entries are skipped and
/// reported in the message instead of aborting the whole clean.
pub async fn clean(cache_type: &CacheType, dry_run: bool) -> Result<CleanResult> {
    let result = |freed_bytes, items_removed, success, message: String| CleanResult {
        cache_type: cache_type.clone(),
        freed_bytes,
        items_removed,
        success,
        message,
        dry_run,
    };

    if !paths::is_supported(cache_type) {
        return Ok(result(0, 0, false, "Unsupported cache type".to_string()));
    }

    let candidates = paths::locations(cache_type);
    let existing = tokio::task::spawn_blocking(move || existing_locations(&candidates)).await?;
    if existing.is_empty() {
        return Ok(result(0, 0, true, "Cache directory does not exist".to_string()));
    }

    if dry_run {
        let stats = tokio::task::spawn_blocking(move || total_stats(&existing)).await?;
        let message = format!("Would free {} bytes ({} items)", stats.bytes, stats.items);
        return Ok(result(stats.bytes, stats.items, true, message));
    }

    let stats = tokio::task::spawn_blocking(move || {
        let mut total = RemovalStats::default();
        for location in &existing {
            total.merge(filesystem::remove_contents(&location.path, location.min_age));
        }
        total
    })
    .await?;

    let mut message = format!("Freed {} bytes ({} items)", stats.freed_bytes, stats.removed);
    if stats.failed > 0 {
        message.push_str(&format!(
            "; skipped {} items that are in use or need administrator rights",
            stats.failed
        ));
    }
    // Partial cleans still count as success; only "nothing could be removed" is a failure.
    let success = stats.failed == 0 || stats.removed > 0;
    Ok(result(stats.freed_bytes, stats.removed, success, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean_unsupported_type() {
        let result = clean(&CacheType::IosBackups, true).await.unwrap();
        assert!(!result.success);
        assert_eq!(result.message, "Unsupported cache type");
    }

    #[tokio::test]
    async fn test_clean_docker_types_are_not_filesystem_caches() {
        let result = clean(&CacheType::DockerImages, true).await.unwrap();
        assert!(!result.success);
    }

    #[tokio::test]
    async fn test_clean_dry_run_does_not_delete() {
        // Supported on every platform; dry run must report without touching anything.
        let result = clean(&CacheType::Npm, true).await.unwrap();
        assert!(result.dry_run);
        assert!(result.success);
        assert!(matches!(result.cache_type, CacheType::Npm));
    }
}
