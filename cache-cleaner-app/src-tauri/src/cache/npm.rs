use std::path::PathBuf;

pub fn get_cache_paths() -> Vec<PathBuf> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return vec![],
    };
    
    vec![
        home.join(".npm"),
        home.join(".npm/_cacache"),
    ]
}

pub fn detect() -> bool {
    get_cache_paths().iter().any(|p| p.exists())
}
