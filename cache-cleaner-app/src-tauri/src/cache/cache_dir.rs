use std::path::PathBuf;

pub fn get_cache_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cache"))
}

pub fn detect() -> bool {
    get_cache_path().map(|p| p.exists()).unwrap_or(false)
}
