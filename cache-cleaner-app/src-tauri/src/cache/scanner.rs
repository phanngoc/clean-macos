use super::{CacheInfo, CacheType};
use crate::cache::docker::{self, DockerBackend};
use crate::cache::paths::{self, Location};
use crate::cache::wsl;
use crate::utils::filesystem::{self, PathStats};
use anyhow::Result;

/// Scans every cache type supported on this platform; returns the non-empty ones.
pub async fn scan_all() -> Result<Vec<CacheInfo>> {
    let scans = paths::supported_types().iter().map(scan_cache);
    let caches = futures::future::join_all(scans)
        .await
        .into_iter()
        .filter_map(|info| info.ok())
        .filter(|info| info.exists && info.size > 0)
        .collect();
    Ok(caches)
}

pub async fn scan_cache(cache_type: &CacheType) -> Result<CacheInfo> {
    if !paths::is_supported(cache_type) {
        return Err(anyhow::anyhow!("{:?} is not supported on this platform", cache_type));
    }
    if docker::is_docker_type(cache_type) {
        let size = docker::reclaimable(cache_type).await?;
        let path = match docker::backend() {
            DockerBackend::Host => "docker".to_string(),
            DockerBackend::Wsl(distro) => format!("docker (WSL: {distro})"),
        };
        return Ok(CacheInfo { cache_type: cache_type.clone(), path, size, exists: true, item_count: 0 });
    }
    if wsl::is_wsl_type(cache_type) {
        let ct = cache_type.clone();
        let (size, item_count, distros) = tokio::task::spawn_blocking(move || wsl::scan(&ct)).await??;
        let path = distros.iter().map(|d| format!("\\\\wsl.localhost\\{d}")).collect::<Vec<_>>().join("\n");
        return Ok(CacheInfo { cache_type: cache_type.clone(), path, size, exists: true, item_count });
    }

    let candidates = paths::locations(cache_type);
    if candidates.is_empty() {
        return Err(anyhow::anyhow!("No {:?} location on this machine", cache_type));
    }

    let cache_type = cache_type.clone();
    tokio::task::spawn_blocking(move || {
        let existing = existing_locations(&candidates);
        let stats = total_stats(&existing);
        let path = if existing.is_empty() {
            candidates[0].path.display().to_string()
        } else {
            existing
                .iter()
                .map(|l| l.path.display().to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };
        CacheInfo {
            cache_type,
            path,
            size: stats.bytes,
            exists: !existing.is_empty(),
            item_count: stats.items,
        }
    })
    .await
    .map_err(Into::into)
}

pub async fn get_size(cache_type: &CacheType) -> Result<u64> {
    Ok(scan_cache(cache_type).await?.size)
}

pub(crate) fn existing_locations(candidates: &[Location]) -> Vec<Location> {
    candidates
        .iter()
        .filter(|l| std::fs::symlink_metadata(&l.path).is_ok())
        .cloned()
        .collect()
}

pub(crate) fn total_stats(locations: &[Location]) -> PathStats {
    locations
        .iter()
        .map(|l| filesystem::path_stats(&l.path, l.min_age))
        .fold(PathStats::default(), |acc, s| PathStats {
            bytes: acc.bytes + s.bytes,
            items: acc.items + s.items,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_unsupported_type_errors() {
        assert!(scan_cache(&CacheType::IosBackups).await.is_err());
    }
}
