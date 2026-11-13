use crate::cache::CacheType;
use dirs::home_dir;
use std::path::{Path, PathBuf};
use std::time::Duration;

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

/// A filesystem location owned by a cache type. Directories are emptied (the directory itself is
/// kept); files are deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub path: PathBuf,
    /// Entries modified more recently than this are kept (they may still be in use).
    pub min_age: Option<Duration>,
}

impl Location {
    pub fn new(path: PathBuf) -> Self {
        Self { path, min_age: None }
    }

    pub fn aged(path: PathBuf, min_age: Duration) -> Self {
        Self { path, min_age: Some(min_age) }
    }
}

/// Temp folders: only touch entries untouched for a day, like Windows Storage Sense.
pub const TEMP_MIN_AGE: Duration = Duration::from_secs(24 * 60 * 60);

/// Cache types this platform knows how to scan and clean, in display order.
pub fn supported_types() -> &'static [CacheType] {
    platform::SUPPORTED
}

pub fn is_supported(cache_type: &CacheType) -> bool {
    supported_types().contains(cache_type)
}

/// Candidate locations for `cache_type` on this platform (may not exist). Empty when unsupported.
pub fn locations(cache_type: &CacheType) -> Vec<Location> {
    if !is_supported(cache_type) {
        return Vec::new();
    }
    platform::locations(cache_type)
}

/// Root of the npm cache (`~/.npm` on macOS, `%LOCALAPPDATA%\npm-cache` on Windows).
pub fn npm_cache_root() -> PathBuf {
    platform::npm_cache_root()
}

/// Chrome "User Data" directory containing `Default`, `Profile N`, ...
pub fn chrome_user_data() -> PathBuf {
    platform::chrome_user_data()
}

fn join_all(base: &Path, names: &[&str]) -> Vec<PathBuf> {
    names.iter().map(|name| base.join(name)).collect()
}

#[cfg(not(windows))]
mod platform {
    use super::*;
    use CacheType::*;

    pub const SUPPORTED: &[CacheType] = &[
        Npm, Chrome, CacheDir, VSCode, Cursor, Safari, Firefox, Arc, Yarn, Pnpm, Pip, CocoaPods,
        Gradle, Cargo, XcodeDerivedData, XcodeArchives, XcodeSimulators,
    ];

    pub fn locations(cache_type: &CacheType) -> Vec<Location> {
        let home = MacPaths::home();
        let paths = match cache_type {
            Npm => vec![npm_cache_root()],
            Chrome => vec![MacPaths::chrome_cache()],
            CacheDir => vec![MacPaths::cache_dir()],
            VSCode => join_all(
                &home.join("Library/Application Support/Code"),
                &["Cache/Cache_Data", "User/workspaceStorage"],
            ),
            Cursor => cursor_paths(&home),
            Safari => vec![MacPaths::safari_cache()],
            Firefox => vec![MacPaths::firefox_profiles()],
            Arc => vec![MacPaths::arc_cache()],
            Yarn => vec![MacPaths::yarn_cache()],
            Pnpm => vec![MacPaths::pnpm_cache()],
            Pip => vec![MacPaths::pip_cache()],
            CocoaPods => vec![MacPaths::cocoapods_cache()],
            Gradle => vec![MacPaths::gradle_cache()],
            Cargo => vec![MacPaths::cargo_cache()],
            XcodeDerivedData => vec![MacPaths::xcode_derived_data()],
            XcodeArchives => vec![MacPaths::xcode_archives()],
            XcodeSimulators => vec![MacPaths::xcode_simulators()],
            _ => Vec::new(),
        };
        paths.into_iter().map(Location::new).collect()
    }

    /// Cursor caches: the todesktop bundle cache, `Caches/Cursor` and safe Application Support dirs.
    fn cursor_paths(home: &Path) -> Vec<PathBuf> {
        let caches_dir = home.join("Library/Caches");
        let mut paths: Vec<PathBuf> = std::fs::read_dir(&caches_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with("com.todesktop.") && e.path().is_dir())
            .map(|e| e.path())
            .collect();
        paths.push(caches_dir.join("Cursor"));
        paths.extend(join_all(
            &home.join("Library/Application Support/Cursor"),
            &["CachedExtensions", "CachedExtensionVSIXs", "logs"],
        ));
        paths
    }

    pub fn npm_cache_root() -> PathBuf {
        MacPaths::home().join(".npm")
    }

    pub fn chrome_user_data() -> PathBuf {
        MacPaths::home().join("Library/Application Support/Google/Chrome")
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use crate::utils::process::command;
    use CacheType::*;

    pub const SUPPORTED: &[CacheType] = &[
        TempFiles, SystemCaches, WindowsUpdate, ThumbnailCache, ErrorReports, CrashDumps,
        ShaderCache, RecycleBin, Chrome, Edge, Firefox, VSCode, Cursor, Npm, Yarn, Pnpm, Pip,
        NuGet, Gradle, Cargo, GoBuild, Uv, CacheDir,
    ];

    /// Per-profile Chromium caches (Chrome, Edge). Everything here is rebuilt on demand.
    const CHROMIUM_PROFILE_CACHES: &[&str] =
        &["Cache", "Code Cache", "GPUCache", "DawnGraphiteCache", "DawnWebGPUCache"];
    /// Caches shared by all profiles, directly under "User Data".
    const CHROMIUM_SHARED_CACHES: &[&str] =
        &["ShaderCache", "GrShaderCache", "GraphiteDawnCache", "GPUPersistentCache"];
    /// Electron editors (VS Code, Cursor) under %APPDATA%. User settings/extensions are not touched.
    const ELECTRON_CACHES: &[&str] = &[
        "Cache", "CachedData", "Code Cache", "GPUCache", "CachedExtensionVSIXs", "logs",
        "DawnGraphiteCache", "DawnWebGPUCache",
    ];

    fn home() -> PathBuf {
        MacPaths::home()
    }

    /// %LOCALAPPDATA%
    fn local() -> PathBuf {
        dirs::data_local_dir().unwrap_or_else(|| home().join("AppData\\Local"))
    }

    /// %APPDATA%
    fn roaming() -> PathBuf {
        dirs::data_dir().unwrap_or_else(|| home().join("AppData\\Roaming"))
    }

    fn local_low() -> PathBuf {
        home().join("AppData\\LocalLow")
    }

    fn env_dir(var: &str, fallback: &str) -> PathBuf {
        std::env::var_os(var).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(fallback))
    }

    fn system_root() -> PathBuf {
        env_dir("SystemRoot", "C:\\Windows")
    }

    fn program_data() -> PathBuf {
        env_dir("ProgramData", "C:\\ProgramData")
    }

    pub fn locations(cache_type: &CacheType) -> Vec<Location> {
        let paths = match cache_type {
            TempFiles => return vec![Location::aged(std::env::temp_dir(), TEMP_MIN_AGE)],
            SystemCaches => return vec![Location::aged(system_root().join("Temp"), TEMP_MIN_AGE)],
            WindowsUpdate => vec![system_root().join("SoftwareDistribution\\Download")],
            ThumbnailCache => explorer_cache_files(&local().join("Microsoft\\Windows\\Explorer")),
            ErrorReports => {
                let wer = ["ReportArchive", "ReportQueue", "Temp"];
                let mut paths = join_all(&local().join("Microsoft\\Windows\\WER"), &wer);
                paths.extend(join_all(&program_data().join("Microsoft\\Windows\\WER"), &wer));
                paths
            }
            CrashDumps => vec![
                local().join("CrashDumps"),
                system_root().join("Minidump"),
                system_root().join("MEMORY.DMP"),
            ],
            ShaderCache => {
                let mut paths = join_all(
                    &local(),
                    &["D3DSCache", "NVIDIA\\DXCache", "NVIDIA\\GLCache", "AMD\\DxCache", "AMD\\GLCache", "AMD\\VkCache"],
                );
                paths.extend(join_all(&local_low(), &["NVIDIA\\PerDriverVersion\\DXCache", "Intel\\ShaderCache"]));
                paths
            }
            RecycleBin => recycle_bin_dirs(),
            Chrome => chromium_caches(&chrome_user_data()),
            Edge => chromium_caches(&local().join("Microsoft\\Edge\\User Data")),
            Firefox => firefox_caches(&local().join("Mozilla\\Firefox\\Profiles")),
            VSCode => join_all(&roaming().join("Code"), ELECTRON_CACHES),
            Cursor => {
                let mut paths = join_all(&roaming().join("Cursor"), ELECTRON_CACHES);
                paths.push(roaming().join("Cursor\\CachedExtensions"));
                paths.push(local().join("cursor-updater"));
                paths
            }
            Npm => vec![npm_cache_root()],
            Yarn => vec![local().join("Yarn\\Cache")],
            Pnpm => vec![local().join("pnpm\\store")],
            Pip => vec![local().join("pip\\cache")],
            NuGet => vec![
                home().join(".nuget\\packages"),
                local().join("NuGet\\v3-cache"),
                local().join("NuGet\\plugins-cache"),
            ],
            Gradle => vec![home().join(".gradle\\caches")],
            Cargo => vec![home().join(".cargo\\registry")],
            GoBuild => vec![std::env::var_os("GOCACHE").map(PathBuf::from).unwrap_or_else(|| local().join("go-build"))],
            Uv => vec![std::env::var_os("UV_CACHE_DIR").map(PathBuf::from).unwrap_or_else(|| local().join("uv\\cache"))],
            CacheDir => vec![home().join(".cache")],
            _ => Vec::new(),
        };
        paths.into_iter().map(Location::new).collect()
    }

    /// Shared caches plus per-profile caches of every profile (dirs holding a `Preferences` file).
    fn chromium_caches(user_data: &Path) -> Vec<PathBuf> {
        let mut paths = join_all(user_data, CHROMIUM_SHARED_CACHES);
        for entry in std::fs::read_dir(user_data).into_iter().flatten().flatten() {
            let profile = entry.path();
            if profile.join("Preferences").is_file() {
                paths.extend(join_all(&profile, CHROMIUM_PROFILE_CACHES));
            }
        }
        paths
    }

    /// %LOCALAPPDATA% Firefox profiles only hold caches; bookmarks etc. live under %APPDATA%.
    fn firefox_caches(profiles: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(profiles)
            .into_iter()
            .flatten()
            .flatten()
            .flat_map(|entry| join_all(&entry.path(), &["cache2", "startupCache", "thumbnails"]))
            .collect()
    }

    /// `thumbcache_*.db` / `iconcache_*.db`, rebuilt by Explorer.
    fn explorer_cache_files(explorer: &Path) -> Vec<PathBuf> {
        std::fs::read_dir(explorer)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                (name.starts_with("thumbcache_") || name.starts_with("iconcache_")) && name.ends_with(".db")
            })
            .map(|entry| entry.path())
            .collect()
    }

    /// The current user's `X:\$Recycle.Bin\<SID>` folder on every drive.
    fn recycle_bin_dirs() -> Vec<PathBuf> {
        let Some(sid) = current_user_sid() else { return Vec::new() };
        (b'A'..=b'Z')
            .map(|drive| PathBuf::from(format!("{}:\\$Recycle.Bin\\{}", drive as char, sid)))
            .filter(|path| path.is_dir())
            .collect()
    }

    fn current_user_sid() -> Option<String> {
        // Output: "DOMAIN\user","S-1-5-21-..."
        let output = command("whoami").args(["/user", "/fo", "csv", "/nh"]).output().ok()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let sid = stdout.trim().rsplit(',').next()?.trim_matches('"');
        sid.starts_with("S-1-").then(|| sid.to_string())
    }

    pub fn npm_cache_root() -> PathBuf {
        local().join("npm-cache")
    }

    pub fn chrome_user_data() -> PathBuf {
        local().join("Google\\Chrome\\User Data")
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_chromium_caches_only_target_real_profiles() {
            let user_data = tempfile::tempdir().unwrap();
            let profile = user_data.path().join("Profile 1");
            std::fs::create_dir(&profile).unwrap();
            std::fs::write(profile.join("Preferences"), "{}").unwrap();
            std::fs::create_dir(user_data.path().join("Safe Browsing")).unwrap();

            let paths = chromium_caches(user_data.path());

            assert!(paths.contains(&profile.join("Cache")));
            assert!(paths.contains(&profile.join("Code Cache")));
            assert!(paths.contains(&user_data.path().join("GrShaderCache")));
            assert!(!paths.iter().any(|p| p.starts_with(user_data.path().join("Safe Browsing"))));
            // Never the profile itself (cookies, history, passwords).
            assert!(!paths.contains(&profile));
        }

        #[test]
        fn test_current_user_sid_parses_whoami() {
            assert!(current_user_sid().is_some_and(|sid| sid.starts_with("S-1-5-")));
        }
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

    #[test]
    fn test_user_data_and_docker_types_have_no_locations() {
        for cache_type in [CacheType::IosBackups, CacheType::DockerImages, CacheType::DockerVolumes] {
            assert!(!is_supported(&cache_type));
            assert!(locations(&cache_type).is_empty());
        }
    }

    #[test]
    fn test_supported_types_are_unique() {
        let types = supported_types();
        for (i, t) in types.iter().enumerate() {
            assert!(!types[i + 1..].contains(t), "{:?} listed twice", t);
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_temp_locations_protect_recent_files() {
        for cache_type in [CacheType::TempFiles, CacheType::SystemCaches] {
            let locations = locations(&cache_type);
            assert!(!locations.is_empty());
            assert!(locations.iter().all(|l| l.min_age == Some(TEMP_MIN_AGE)));
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_editor_locations_skip_user_settings_and_extensions() {
        for cache_type in [CacheType::VSCode, CacheType::Cursor] {
            for location in locations(&cache_type) {
                let name = location.path.file_name().unwrap().to_string_lossy().to_lowercase();
                assert!(name != "user" && name != "extensions", "{:?}", location.path);
            }
        }
    }
}
