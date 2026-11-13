//! Headless front-end for the cache cleaner (no webview needed).
//!
//!   cache-cleaner-cli scan
//!   cache-cleaner-cli clean [--dry-run] <type>...
//!
//! `<type>` is any name printed by `scan`/`types`, case-insensitive (e.g. `tempfiles`, `chrome`).

use cache_cleaner::cache::{cleaner, paths, scanner, CacheType};
use cache_cleaner::utils::permissions;
use std::process::ExitCode;

const USAGE: &str = "usage:
  cache-cleaner-cli scan                          list caches on this machine and their size
  cache-cleaner-cli types                         list cache types supported on this platform
  cache-cleaner-cli clean [--dry-run] <type>...   empty the given caches";

#[tokio::main]
async fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("scan") => scan().await,
        Some("types") => {
            for cache_type in paths::supported_types() {
                println!("{:?}", cache_type);
            }
            ExitCode::SUCCESS
        }
        Some("clean") => clean(&args[1..]).await,
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

async fn scan() -> ExitCode {
    let mut caches = match scanner::scan_all().await {
        Ok(caches) => caches,
        Err(err) => {
            eprintln!("scan failed: {err}");
            return ExitCode::FAILURE;
        }
    };
    caches.sort_by(|a, b| b.size.cmp(&a.size));

    println!("{:<16} {:>10} {:>9}  PATH", "TYPE", "SIZE", "ITEMS");
    for info in &caches {
        let mut lines = info.path.lines();
        let first = lines.next().unwrap_or_default();
        let more = lines.count();
        let suffix = if more > 0 { format!("  (+{more} more)") } else { String::new() };
        println!(
            "{:<16} {:>10} {:>9}  {first}{suffix}",
            format!("{:?}", info.cache_type),
            format_size(info.size),
            info.item_count
        );
    }
    let total: u64 = caches.iter().map(|c| c.size).sum();
    println!("{:<16} {:>10}", "TOTAL", format_size(total));
    warn_if_unprivileged();
    ExitCode::SUCCESS
}

async fn clean(args: &[String]) -> ExitCode {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let types: Result<Vec<CacheType>, String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(|a| CacheType::from_str(a))
        .collect();
    let types = match types {
        Ok(types) if !types.is_empty() => types,
        Ok(_) => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::from(2);
        }
    };

    let mut total = 0u64;
    let mut all_ok = true;
    for cache_type in &types {
        match cleaner::clean(cache_type, dry_run).await {
            Ok(result) => {
                total += result.freed_bytes;
                all_ok &= result.success;
                let status = if result.success { "ok" } else { "FAILED" };
                println!(
                    "{:<16} {:>10}  {status}: {}",
                    format!("{:?}", cache_type),
                    format_size(result.freed_bytes),
                    result.message
                );
            }
            Err(err) => {
                all_ok = false;
                println!("{:<16} {:>10}  FAILED: {err}", format!("{:?}", cache_type), "-");
            }
        }
    }
    let verb = if dry_run { "Would free" } else { "Freed" };
    println!("{verb} {} in total", format_size(total));
    if all_ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

fn warn_if_unprivileged() {
    if cfg!(windows) && !permissions::has_full_disk_access() {
        eprintln!(
            "note: not running as administrator; SystemCaches, WindowsUpdate and system crash \
             dumps/error reports are only partially accessible."
        );
    }
}

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{size:.1} {}", UNITS[unit]) }
}
