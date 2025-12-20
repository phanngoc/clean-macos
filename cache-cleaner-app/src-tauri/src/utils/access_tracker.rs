use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderAccessInfo {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub last_accessed: Option<u64>,  // seconds since epoch
    pub last_modified: Option<u64>,
    pub item_count: usize,
}

pub fn get_access_info(path: &PathBuf) -> Result<FolderAccessInfo> {
    let metadata = std::fs::metadata(path)?;
    
    let last_accessed = metadata.accessed().ok().and_then(|t| {
        t.duration_since(SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs())
    });
    
    let last_modified = metadata.modified().ok().and_then(|t| {
        t.duration_since(SystemTime::UNIX_EPOCH).ok().map(|d| d.as_secs())
    });

    Ok(FolderAccessInfo {
        path: path.clone(),
        size_bytes: 0,
        last_accessed,
        last_modified,
        item_count: 0,
    })
}

pub fn days_since_access(info: &FolderAccessInfo) -> Option<u64> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    
    info.last_accessed.map(|accessed| (now - accessed) / 86400)
}
