use anyhow::Result;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, SystemTime};
use walkdir::{DirEntry, WalkDir};

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

/// Size and entry count of everything below `path` (or of `path` itself when it is a file).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PathStats {
    pub bytes: u64,
    pub items: usize,
}

/// Outcome of a best-effort removal.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RemovalStats {
    pub freed_bytes: u64,
    pub removed: usize,
    /// Entries that could not be read or deleted (locked, in use, access denied).
    pub failed: usize,
}

impl RemovalStats {
    pub fn merge(&mut self, other: RemovalStats) {
        self.freed_bytes += other.freed_bytes;
        self.removed += other.removed;
        self.failed += other.failed;
    }
}

/// Entries modified less than `min_age` ago are left alone.
fn old_enough(entry: &DirEntry, min_age: Option<Duration>) -> bool {
    let Some(min_age) = min_age else { return true };
    let modified = entry.metadata().ok().and_then(|m| m.modified().ok());
    match modified.and_then(|t| SystemTime::now().duration_since(t).ok()) {
        Some(age) => age >= min_age,
        // Unknown or future mtime: treat as in use.
        None => false,
    }
}

fn min_depth(path: &Path) -> usize {
    if path.is_dir() { 1 } else { 0 }
}

/// Stats for what `remove_contents` would delete. Symlinks/junctions are never followed.
pub fn path_stats(path: &Path, min_age: Option<Duration>) -> PathStats {
    let mut stats = PathStats::default();
    for entry in WalkDir::new(path).min_depth(min_depth(path)).into_iter().filter_map(|e| e.ok()) {
        if !old_enough(&entry, min_age) {
            continue;
        }
        stats.items += 1;
        if entry.file_type().is_file() {
            stats.bytes += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    stats
}

/// Deletes everything inside directory `path` (the directory itself is kept), or `path` itself
/// when it is a file. Keeps going past locked / access-denied entries and reports them in
/// `failed`. Symlinks and junctions are unlinked, never followed.
pub fn remove_contents(path: &Path, min_age: Option<Duration>) -> RemovalStats {
    let mut stats = RemovalStats::default();
    if fs::symlink_metadata(path).is_err() {
        return stats;
    }

    let walker = WalkDir::new(path).min_depth(min_depth(path)).contents_first(true);
    for entry in walker {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                stats.failed += 1;
                continue;
            }
        };
        if !old_enough(&entry, min_age) {
            continue;
        }
        let file_type = entry.file_type();
        if file_type.is_dir() {
            // Fails while a skipped child remains; that child is already counted as failed/kept.
            if fs::remove_dir(entry.path()).is_ok() {
                stats.removed += 1;
            }
            continue;
        }
        let len = if file_type.is_file() {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };
        match remove_non_dir(entry.path()) {
            Ok(()) => {
                stats.freed_bytes += len;
                stats.removed += 1;
            }
            Err(_) => stats.failed += 1,
        }
    }
    stats
}

/// Removes a file or a symlink/junction (which may point at a directory).
fn remove_non_dir(path: &Path) -> io::Result<()> {
    let err = match fs::remove_file(path) {
        Ok(()) => return Ok(()),
        Err(err) => err,
    };
    // Directory symlinks/junctions on Windows must be removed with remove_dir.
    if fs::remove_dir(path).is_ok() {
        return Ok(());
    }
    // Windows refuses to delete read-only files.
    let mut perms = fs::symlink_metadata(path)?.permissions();
    if err.kind() != io::ErrorKind::PermissionDenied || !perms.readonly() {
        return Err(err);
    }
    #[allow(clippy::permissions_set_readonly_false)]
    perms.set_readonly(false);
    fs::set_permissions(path, perms)?;
    fs::remove_file(path)
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
    fn test_path_stats_counts_nested_entries_and_bytes() {
        let dir = create_test_dir();
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(dir.path(), "root.txt", b"root");
        create_test_file(&subdir, "nested.txt", b"nested");

        let stats = path_stats(dir.path(), None);
        assert_eq!(stats, PathStats { bytes: 10, items: 3 }); // subdir + 2 files
    }

    #[test]
    fn test_path_stats_single_file() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "thumbcache_32.db", b"12345");
        assert_eq!(path_stats(&file, None), PathStats { bytes: 5, items: 1 });
    }

    #[test]
    fn test_remove_contents_keeps_root() {
        let dir = create_test_dir();
        create_test_file(dir.path(), "file1.txt", b"test");
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_file(&subdir, "nested.txt", b"nested");

        let stats = remove_contents(dir.path(), None);

        assert_eq!(stats, RemovalStats { freed_bytes: 10, removed: 3, failed: 0 });
        assert!(!dir.path().join("file1.txt").exists());
        assert!(!subdir.exists());
        assert!(dir.path().exists());
    }

    #[test]
    fn test_remove_contents_single_file() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "iconcache_16.db", b"abc");
        let stats = remove_contents(&file, None);
        assert_eq!(stats, RemovalStats { freed_bytes: 3, removed: 1, failed: 0 });
        assert!(!file.exists());
    }

    #[test]
    fn test_remove_contents_nonexistent() {
        let stats = remove_contents(Path::new("/nonexistent/path"), None);
        assert_eq!(stats, RemovalStats::default());
    }

    #[test]
    fn test_remove_contents_skips_recent_entries() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "in-use.tmp", b"busy");

        let one_day = Some(Duration::from_secs(24 * 3600));
        assert_eq!(path_stats(dir.path(), one_day), PathStats::default());
        let stats = remove_contents(dir.path(), one_day);

        assert_eq!(stats, RemovalStats::default());
        assert!(file.exists());
    }

    #[test]
    fn test_remove_contents_deletes_read_only_files() {
        let dir = create_test_dir();
        let file = create_test_file(dir.path(), "readonly.bin", b"ro");
        let mut perms = fs::metadata(&file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file, perms).unwrap();

        let stats = remove_contents(dir.path(), None);

        assert_eq!(stats.failed, 0);
        assert!(!file.exists());
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
