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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_returns_valid_path() {
        let home = MacPaths::home();
        assert!(!home.as_os_str().is_empty());
    }

    #[test]
    fn test_browser_cache_paths() {
        let chrome = MacPaths::chrome_cache();
        assert!(chrome.to_string_lossy().contains("Library/Caches/Google/Chrome"));

        let safari = MacPaths::safari_cache();
        assert!(safari.to_string_lossy().contains("com.apple.Safari"));

        let firefox = MacPaths::firefox_profiles();
        assert!(firefox.to_string_lossy().contains("Firefox"));

        let arc = MacPaths::arc_cache();
        assert!(arc.to_string_lossy().contains("company.thebrowser.Browser"));
    }

    #[test]
    fn test_package_manager_paths() {
        let npm = MacPaths::npm_cache();
        assert!(npm.to_string_lossy().contains(".npm"));

        let yarn = MacPaths::yarn_cache();
        assert!(yarn.to_string_lossy().contains("Yarn"));

        let pip = MacPaths::pip_cache();
        assert!(pip.to_string_lossy().contains("pip"));

        let cargo = MacPaths::cargo_cache();
        assert!(cargo.to_string_lossy().contains(".cargo"));
    }

    #[test]
    fn test_xcode_paths() {
        let derived = MacPaths::xcode_derived_data();
        assert!(derived.to_string_lossy().contains("DerivedData"));

        let archives = MacPaths::xcode_archives();
        assert!(archives.to_string_lossy().contains("Archives"));

        let simulators = MacPaths::xcode_simulators();
        assert!(simulators.to_string_lossy().contains("CoreSimulator"));
    }

    #[test]
    fn test_system_paths() {
        let tmp = MacPaths::tmp();
        assert_eq!(tmp, PathBuf::from("/tmp"));

        let system_caches = MacPaths::system_caches();
        assert_eq!(system_caches, PathBuf::from("/Library/Caches"));

        let system_logs = MacPaths::system_logs();
        assert_eq!(system_logs, PathBuf::from("/var/log"));
    }

    #[test]
    fn test_is_system_path() {
        assert!(MacPaths::is_system_path(&PathBuf::from("/System/Library")));
        assert!(MacPaths::is_system_path(&PathBuf::from("/usr/bin")));
        assert!(MacPaths::is_system_path(&PathBuf::from("/bin/bash")));
        assert!(MacPaths::is_system_path(&PathBuf::from("/private/var/db/test")));
        
        assert!(!MacPaths::is_system_path(&PathBuf::from("/Users/test")));
        assert!(!MacPaths::is_system_path(&PathBuf::from("/Applications")));
        assert!(!MacPaths::is_system_path(&PathBuf::from("/tmp")));
    }

    #[test]
    fn test_all_cache_paths_not_empty() {
        let paths = MacPaths::all_cache_paths();
        assert!(!paths.is_empty());
        assert!(paths.len() >= 10);
        
        for (name, path) in &paths {
            assert!(!name.is_empty());
            assert!(!path.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_user_directories() {
        let trash = MacPaths::trash();
        assert!(trash.to_string_lossy().contains(".Trash"));

        let downloads = MacPaths::downloads();
        assert!(downloads.to_string_lossy().contains("Downloads"));

        let documents = MacPaths::documents();
        assert!(documents.to_string_lossy().contains("Documents"));
    }

    #[test]
    fn test_ios_paths() {
        let backups = MacPaths::ios_backups();
        assert!(backups.to_string_lossy().contains("MobileSync/Backup"));
    }
}
