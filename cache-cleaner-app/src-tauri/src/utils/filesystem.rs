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

pub async fn calculate_file_size(path: &Path) -> Result<u64> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || calculate_file_size_sync(&path))
        .await?
}

pub fn calculate_file_size_sync(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }
    Ok(std::fs::metadata(path)?.len())
}

pub fn remove_file(path: &Path) -> Result<()> {
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_dir() -> TempDir {
        tempfile::tempdir().unwrap()
    }

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        path
    }

    #[test]
    fn test_calculate_dir_size_sync_empty() {
        let dir = create_test_dir();
        let size = calculate_dir_size_sync(dir.path()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_sync_with_files() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "file1.txt", b"hello");
        create_test_file(dir.path(), "file2.txt", b"world!");
        
        let size = calculate_dir_size_sync(dir.path()).unwrap();
        assert_eq!(size, 11); // 5 + 6 bytes
    }

    #[test]
    fn test_calculate_dir_size_sync_nested() {
        let dir = create_test_dir();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        create_test_file(dir.path(), "root.txt", b"root");
        create_test_file(&subdir, "nested.txt", b"nested");
        
        let size = calculate_dir_size_sync(dir.path()).unwrap();
        assert_eq!(size, 10); // 4 + 6 bytes
    }

    #[test]
    fn test_calculate_dir_size_sync_nonexistent() {
        let size = calculate_dir_size_sync(Path::new("/nonexistent/path")).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_count_items_empty() {
        let dir = create_test_dir();
        let count = count_items(dir.path()).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_items_with_files() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "file1.txt", b"a");
        create_test_file(dir.path(), "file2.txt", b"b");
        
        let count = count_items(dir.path()).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_count_items_nested() {
        let dir = create_test_dir();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        
        create_test_file(dir.path(), "root.txt", b"r");
        create_test_file(&subdir, "nested.txt", b"n");
        
        let count = count_items(dir.path()).unwrap();
        assert_eq!(count, 3); // subdir + 2 files
    }

    #[test]
    fn test_remove_dir_contents() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "file1.txt", b"test");
        
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "nested.txt", b"nested");
        
        assert!(dir.path().join("file1.txt").exists());
        assert!(subdir.exists());
        
        remove_dir_contents(dir.path()).unwrap();
        
        assert!(!dir.path().join("file1.txt").exists());
        assert!(!subdir.exists());
        assert!(dir.path().exists()); // Parent dir still exists
    }

    #[test]
    fn test_remove_dir_contents_nonexistent() {
        let result = remove_dir_contents(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_calculate_file_size_sync() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "test.txt", b"hello world");
        
        let size = calculate_file_size_sync(&file).unwrap();
        assert_eq!(size, 11);
    }

    #[test]
    fn test_calculate_file_size_sync_nonexistent() {
        let size = calculate_file_size_sync(Path::new("/nonexistent/file.txt")).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_remove_file() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "to_delete.txt", b"delete me");
        
        assert!(file.exists());
        remove_file(&file).unwrap();
        assert!(!file.exists());
    }

    #[test]
    fn test_remove_file_nonexistent() {
        let result = remove_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_calculate_dir_size_async() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "async_test.txt", b"async content");
        
        let size = calculate_dir_size(dir.path()).await.unwrap();
        assert_eq!(size, 13);
    }

    #[tokio::test]
    async fn test_calculate_file_size_async() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "async_file.txt", b"async file");
        
        let size = calculate_file_size(&file).await.unwrap();
        assert_eq!(size, 10);
    }
}
