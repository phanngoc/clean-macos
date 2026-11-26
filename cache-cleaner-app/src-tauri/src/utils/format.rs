//! Human-readable sizes and durations for CLI / log output.

/// `1536` → `"1.5 KB"` (binary units, one decimal).
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{value:.1} {}", UNITS[unit]) }
}

/// Seconds elapsed → `"just now"`, `"5 min ago"`, `"3 h ago"`, `"2 days ago"`.
pub fn ago(seconds: u64) -> String {
    match seconds {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", seconds / 60),
        3600..=86_399 => format!("{} h ago", seconds / 3600),
        86_400..=172_799 => "1 day ago".to_string(),
        _ => format!("{} days ago", seconds / 86_400),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(size(0), "0 B");
        assert_eq!(size(1023), "1023 B");
        assert_eq!(size(1536), "1.5 KB");
        assert_eq!(size(5 * 1024 * 1024 * 1024), "5.0 GB");
    }

    #[test]
    fn test_ago() {
        assert_eq!(ago(10), "just now");
        assert_eq!(ago(300), "5 min ago");
        assert_eq!(ago(7200), "2 h ago");
        assert_eq!(ago(90_000), "1 day ago");
        assert_eq!(ago(10 * 86_400), "10 days ago");
    }
}
