use super::{CacheType, CleanResult};
use crate::cache::{docker, paths, wsl};
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

    if docker::is_docker_type(cache_type) {
        if dry_run {
            let bytes = docker::reclaimable(cache_type).await?;
            return Ok(result(bytes, 0, true, format!("Would reclaim {} bytes", bytes)));
        }
        let r = docker::prune_cache_type(cache_type).await?;
        let items = r.containers_removed + r.images_removed + r.volumes_removed + r.networks_removed;
        return Ok(result(r.freed_bytes, items, r.success, r.message));
    }

    if wsl::is_wsl_type(cache_type) {
        let ct = cache_type.clone();
        let (bytes, files, distros) = tokio::task::spawn_blocking(move || {
            if dry_run { wsl::scan(&ct) } else { wsl::clean(&ct) }
        })
        .await??;
        let verb = if dry_run { "Would free" } else { "Freed (and trimmed)" };
        let message = format!("{verb} {bytes} bytes ({files} files) in WSL: {}", distros.join(", "));
        return Ok(result(bytes, files, true, message));
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
    async fn test_clean_dry_run_reports_without_failing() {
        // Supported on every platform.
        let result = clean(&CacheType::Npm, true).await.unwrap();
        assert!(result.dry_run);
        assert!(result.success);
        assert!(matches!(result.cache_type, CacheType::Npm));
    }
}
