use std::path::PathBuf;
use dirs::home_dir;

pub struct MacPaths;

impl MacPaths {
    pub fn home() -> PathBuf {
        home_dir().unwrap_or_else(|| PathBuf::from("/"))
    }

    // Browser Caches
    pub fn chrome_cache() -> PathBuf {
        Self::home().join("Library/Caches/Google/Chrome")
    }

    pub fn chrome_default_cache() -> PathBuf {
        Self::home().join("Library/Caches/Google/Chrome/Default/Cache")
    }

    pub fn safari_cache() -> PathBuf {
        Self::home().join("Library/Caches/com.apple.Safari")
    }

    pub fn firefox_profiles() -> PathBuf {
        Self::home().join("Library/Caches/Firefox/Profiles")
    }

    pub fn arc_cache() -> PathBuf {
        Self::home().join("Library/Caches/company.thebrowser.Browser")
    }

    // Package Manager Caches
    pub fn npm_cache() -> PathBuf {
        Self::home().join(".npm/_cacache")
    }

    pub fn yarn_cache() -> PathBuf {
        Self::home().join("Library/Caches/Yarn")
    }

    pub fn pnpm_cache() -> PathBuf {
        Self::home().join("Library/pnpm/store")
    }

    pub fn pip_cache() -> PathBuf {
        Self::home().join(".cache/pip")
    }

    pub fn cocoapods_cache() -> PathBuf {
        Self::home().join("Library/Caches/CocoaPods")
    }

    pub fn gradle_cache() -> PathBuf {
        Self::home().join(".gradle/caches")
    }

    pub fn cargo_cache() -> PathBuf {
        Self::home().join(".cargo/registry")
    }

    // Development Tools
    pub fn xcode_derived_data() -> PathBuf {
        Self::home().join("Library/Developer/Xcode/DerivedData")
    }

    pub fn xcode_archives() -> PathBuf {
        Self::home().join("Library/Developer/Xcode/Archives")
    }

    pub fn xcode_simulators() -> PathBuf {
        Self::home().join("Library/Developer/CoreSimulator/Devices")
    }

    // System Caches
    pub fn user_caches() -> PathBuf {
        Self::home().join("Library/Caches")
    }

    pub fn system_caches() -> PathBuf {
        PathBuf::from("/Library/Caches")
    }

    pub fn user_logs() -> PathBuf {
        Self::home().join("Library/Logs")
    }

    pub fn system_logs() -> PathBuf {
        PathBuf::from("/var/log")
    }

    pub fn tmp() -> PathBuf {
        PathBuf::from("/tmp")
    }

    pub fn var_folders() -> PathBuf {
        PathBuf::from("/private/var/folders")
    }

    // User Directories
    pub fn trash() -> PathBuf {
        Self::home().join(".Trash")
    }

    pub fn downloads() -> PathBuf {
        Self::home().join("Downloads")
    }

    pub fn documents() -> PathBuf {
        Self::home().join("Documents")
    }

    // iOS/Mobile
    pub fn ios_backups() -> PathBuf {
        Self::home().join("Library/Application Support/MobileSync/Backup")
    }

    pub fn mail_downloads() -> PathBuf {
        Self::home().join("Library/Containers/com.apple.mail/Data/Library/Mail Downloads")
    }

    // Applications
    pub fn applications() -> PathBuf {
        PathBuf::from("/Applications")
    }

    // Generic cache directory
    pub fn cache_dir() -> PathBuf {
        Self::home().join(".cache")
    }

    // Check if path is system-protected
    pub fn is_system_path(path: &PathBuf) -> bool {
        let path_str = path.to_string_lossy();
        let system_paths = [
            "/System",
            "/usr",
            "/bin", 
            "/sbin",
            "/private/var/db",
            "/private/var/root",
        ];
        
        system_paths.iter().any(|p| path_str.starts_with(p))
    }

    // Get all cache paths as a vector
    pub fn all_cache_paths() -> Vec<(String, PathBuf)> {
        vec![
            ("Chrome Cache".to_string(), Self::chrome_cache()),
            ("Safari Cache".to_string(), Self::safari_cache()),
            ("Firefox Profiles".to_string(), Self::firefox_profiles()),
            ("Arc Cache".to_string(), Self::arc_cache()),
            ("npm Cache".to_string(), Self::npm_cache()),
            ("Yarn Cache".to_string(), Self::yarn_cache()),
            ("pnpm Cache".to_string(), Self::pnpm_cache()),
            ("pip Cache".to_string(), Self::pip_cache()),
            ("CocoaPods Cache".to_string(), Self::cocoapods_cache()),
            ("Gradle Cache".to_string(), Self::gradle_cache()),
            ("Cargo Cache".to_string(), Self::cargo_cache()),
            ("Xcode DerivedData".to_string(), Self::xcode_derived_data()),
            ("Xcode Archives".to_string(), Self::xcode_archives()),
            ("Xcode Simulators".to_string(), Self::xcode_simulators()),
            ("User Caches".to_string(), Self::user_caches()),
            ("Cache Directory".to_string(), Self::cache_dir()),
        ]
    }
}
