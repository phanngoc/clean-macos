//! Headless front-end for the cache cleaner (no webview needed).
//!
//!   cache-cleaner-cli scan
//!   cache-cleaner-cli clean [--dry-run] <type>...
//!   cache-cleaner-cli artifacts [--idle-days N] [--sweep-days N] [--delete] <root>...
//!
//! `<type>` is any name printed by `scan`/`types`, case-insensitive (e.g. `tempfiles`, `chrome`).
//! An artifacts `<root>` prefixed with `wsl:` (e.g. `wsl:/home/me`) is scanned inside the WSL distro.

use cache_cleaner::cache::artifacts::{self, Artifact};
use cache_cleaner::cache::{cleaner, paths, scanner, wsl, CacheType};
use cache_cleaner::utils::permissions;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

const USAGE: &str = "usage:
  cache-cleaner-cli scan                          list caches on this machine and their size
  cache-cleaner-cli types                         list cache types supported on this platform
  cache-cleaner-cli clean [--dry-run] <type>...   empty the given caches
  cache-cleaner-cli artifacts [--idle-days 30] [--sweep-days 0] [--delete] <root>...
      build output (node_modules, target, .next, .venv, CACHEDIR.TAG dirs) of projects with no
      file changed for --idle-days; --sweep-days N also trims files unread for N days inside
      active CACHEDIR.TAG dirs (shared cargo targets). Lists only unless --delete.";

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
        Some("artifacts") => artifacts_cmd(&args[1..]),
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

fn artifacts_cmd(args: &[String]) -> ExitCode {
    let (mut idle_days, mut sweep_days, mut delete) = (30u64, 0u64, false);
    let mut roots = Vec::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--delete" => delete = true,
            "--idle-days" | "--sweep-days" => {
                let Some(value) = it.next().and_then(|v| v.parse().ok()) else {
                    eprintln!("{arg} needs a number of days\n{USAGE}");
                    return ExitCode::from(2);
                };
                if arg == "--idle-days" { idle_days = value } else { sweep_days = value }
            }
            _ => roots.push(arg.clone()),
        }
    }
    if roots.is_empty() || idle_days == 0 {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
    let days = |n: u64| Duration::from_secs(n * 24 * 3600);
    let (wsl_roots, local_roots): (Vec<String>, Vec<String>) =
        roots.into_iter().partition(|r| r.starts_with("wsl:"));

    let mut rows: Vec<Artifact> = Vec::new();
    let mut freed = 0u64;
    if !local_roots.is_empty() {
        let local: Vec<PathBuf> = local_roots.iter().map(PathBuf::from).collect();
        for artifact in artifacts::find_artifacts(&local, days(idle_days), (sweep_days > 0).then(|| days(sweep_days))) {
            if delete {
                freed += artifacts::remove_artifact(&artifact).freed_bytes;
            }
            rows.push(artifact);
        }
    }
    if !wsl_roots.is_empty() {
        let distros = wsl::cleanable_distros();
        let Some(distro) = distros.iter().find(|d| d.is_default).or(distros.first()) else {
            eprintln!("no WSL 2 distribution found");
            return ExitCode::FAILURE;
        };
        let roots: Vec<String> = wsl_roots.iter().map(|r| r["wsl:".len()..].to_string()).collect();
        let script = artifacts::wsl_script(&roots, idle_days, sweep_days, delete);
        let output = match wsl::run_script(&distro.name, &script) {
            Ok(output) => output,
            Err(err) => {
                eprintln!("WSL scan failed: {err}");
                return ExitCode::FAILURE;
            }
        };
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            if let [size, kind, path, project] = fields[..] {
                let size_bytes = size.parse().unwrap_or(0);
                if delete {
                    freed += size_bytes;
                }
                rows.push(Artifact {
                    path: PathBuf::from(format!("wsl:{path}")),
                    kind: kind.to_string(),
                    size_bytes,
                    project: PathBuf::from(project),
                    unread_for: (kind == artifacts::SWEEP_KIND).then(|| days(sweep_days)),
                });
            }
        }
        if delete {
            wsl::trim(&distro.name);
        }
    }

    rows.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    for row in &rows {
        println!("{:>10}  {:<13} {}", format_size(row.size_bytes), row.kind, row.path.display());
    }
    let total: u64 = rows.iter().map(|r| r.size_bytes).sum();
    if delete {
        println!("Freed {} of {} found", format_size(freed), format_size(total));
    } else {
        println!("{} in {} entries (re-run with --delete to remove)", format_size(total), rows.len());
    }
    ExitCode::SUCCESS
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
