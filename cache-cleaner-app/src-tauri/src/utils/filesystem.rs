use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

pub async fn calculate_dir_size(path: &Path) -> Result<u64> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || calculate_dir_size_sync(&path))
        .await?
}

pub fn calculate_dir_size_sync(path: &Path) -> Result<u64> {
    let mut size = 0u64;
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(size)
}

pub fn count_items(path: &Path) -> Result<usize> {
    Ok(WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .count()
        .saturating_sub(1))
}

pub fn remove_dir_contents(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        } else {
            std::fs::remove_file(&path)?;
        }
    }
    Ok(())
}
