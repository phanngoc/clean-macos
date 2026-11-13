use crate::utils::process::command;
use anyhow::Result;

#[cfg(not(windows))]
pub fn is_chrome_running() -> Result<bool> {
    let output = command("pgrep").args(["-x", "Google Chrome"]).output()?;
    Ok(output.status.success())
}

#[cfg(windows)]
pub fn is_chrome_running() -> Result<bool> {
    let output = command("tasklist")
        .args(["/FI", "IMAGENAME eq chrome.exe", "/FO", "CSV", "/NH"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .to_ascii_lowercase()
        .contains("\"chrome.exe\""))
}

/// macOS: Full Disk Access granted. Windows: running elevated (administrator), which is what
/// system locations such as `C:\Windows\Temp` and the Windows Update cache require.
pub fn has_full_disk_access() -> bool {
    // Probe a directory only readable with that privilege.
    #[cfg(not(windows))]
    let protected_path = dirs::home_dir()
        .map(|h| h.join("Library/Safari"))
        .unwrap_or_default();
    #[cfg(windows)]
    let protected_path = std::path::PathBuf::from(
        std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into()),
    )
    .join("System32\\config");
    std::fs::read_dir(protected_path).is_ok()
}

pub fn can_access_home() -> bool {
    dirs::home_dir()
        .map(|h| std::fs::read_dir(h).is_ok())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_chrome_running_returns_result() {
        // Just verify it doesn't panic and returns a valid Result
        let result = is_chrome_running();
        assert!(result.is_ok());
    }

    #[test]
    fn test_has_full_disk_access_returns_bool() {
        // Just verify it returns a boolean without panicking
        let _result = has_full_disk_access();
    }

    #[test]
    fn test_can_access_home() {
        // Home directory should be accessible in test environment
        let result = can_access_home();
        assert!(result);
    }
}
