use anyhow::Result;
use std::process::Command;

pub fn is_chrome_running() -> Result<bool> {
    let output = Command::new("pgrep")
        .args(["-x", "Google Chrome"])
        .output()?;
    Ok(output.status.success())
}

pub fn has_full_disk_access() -> bool {
    // Check by trying to access a protected directory
    let protected_path = dirs::home_dir()
        .map(|h| h.join("Library/Safari"))
        .unwrap_or_default();
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
