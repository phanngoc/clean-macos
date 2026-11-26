//! Single-purpose commands: `scan`, `types`, `clean`, `artifacts`.

use cache_cleaner::cache::artifacts::{self, Artifact};
use cache_cleaner::cache::{cleaner, paths, scanner, wsl, CacheType};
use cache_cleaner::utils::format::size;
use cache_cleaner::utils::permissions;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

pub async fn scan() -> ExitCode {
    let mut caches = match scanner::scan_all().await {
        Ok(caches) => caches,
        Err(err) => {
            eprintln!("scan failed: {err}");
            return ExitCode::FAILURE;
        }
    };
    caches.sort_by_key(|c| std::cmp::Reverse(c.size));

    println!("{:<16} {:>10} {:>9}  PATH", "TYPE", "SIZE", "ITEMS");
    for info in &caches {
        let mut lines = info.path.lines();
        let first = lines.next().unwrap_or_default();
        let more = lines.count();
        let suffix = if more > 0 { format!("  (+{more} more)") } else { String::new() };
        println!(
            "{:<16} {:>10} {:>9}  {first}{suffix}",
            format!("{:?}", info.cache_type),
            size(info.size),
            info.item_count
        );
    }
    let total: u64 = caches.iter().map(|c| c.size).sum();
    println!("{:<16} {:>10}", "TOTAL", size(total));
    warn_if_unprivileged();
    ExitCode::SUCCESS
}

pub fn types() -> ExitCode {
    for cache_type in paths::supported_types() {
        println!("{:?}", cache_type);
    }
    ExitCode::SUCCESS
}

pub async fn clean(names: &[String], dry_run: bool) -> ExitCode {
    let types: Result<Vec<CacheType>, String> = names.iter().map(|a| CacheType::from_str(a)).collect();
    let types = match types {
        Ok(types) => types,
        Err(err) => {
            eprintln!("{err} (see `cache-cleaner-cli types`)");
            return ExitCode::from(2);
        }
    };

    let mut total = 0u64;
    let mut all_ok = true;
    for cache_type in &types {
        let name = format!("{:?}", cache_type);
        match cleaner::clean(cache_type, dry_run).await {
            Ok(result) => {
                total += result.freed_bytes;
                all_ok &= result.success;
                let status = if result.success { "ok" } else { "FAILED" };
                println!("{name:<16} {:>10}  {status}: {}", size(result.freed_bytes), result.message);
            }
            Err(err) => {
                all_ok = false;
                println!("{name:<16} {:>10}  FAILED: {err}", "-");
            }
        }
    }
    let verb = if dry_run { "Would free" } else { "Freed" };
    println!("{verb} {} in total", size(total));
    if all_ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

pub fn artifacts(roots: Vec<String>, idle_days: u64, sweep_days: u64, delete: bool) -> ExitCode {
    let days = |n: u64| Duration::from_secs(n * 24 * 3600);
    let (wsl_roots, local_roots): (Vec<String>, Vec<String>) =
        roots.into_iter().partition(|r| r.starts_with(artifacts::WSL_PREFIX));

    let mut rows: Vec<Artifact> = Vec::new();
    let mut freed = 0u64;
    if !local_roots.is_empty() {
        let local: Vec<PathBuf> = local_roots.iter().map(PathBuf::from).collect();
        let sweep = (sweep_days > 0).then(|| days(sweep_days));
        for artifact in artifacts::find_artifacts(&local, days(idle_days), sweep) {
            if delete {
                freed += artifacts::remove_artifact(&artifact).freed_bytes;
            }
            rows.push(artifact);
        }
    }
    if !wsl_roots.is_empty() {
        let roots: Vec<String> =
            wsl_roots.iter().map(|r| r[artifacts::WSL_PREFIX.len()..].to_string()).collect();
        match artifacts::find_wsl_artifacts(&roots, idle_days, sweep_days, delete) {
            Ok((distro, found)) => {
                if delete {
                    freed += found.iter().map(|a| a.size_bytes).sum::<u64>();
                    wsl::trim(&distro);
                }
                rows.extend(found);
            }
            Err(err) => {
                eprintln!("WSL scan failed: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    rows.sort_by_key(|r| std::cmp::Reverse(r.size_bytes));
    for row in &rows {
        println!("{:>10}  {:<13} {}", size(row.size_bytes), row.kind, row.path.display());
    }
    let total: u64 = rows.iter().map(|r| r.size_bytes).sum();
    if delete {
        println!("Freed {} of {} found", size(freed), size(total));
    } else {
        println!("{} in {} entries (re-run with --delete to remove)", size(total), rows.len());
    }
    ExitCode::SUCCESS
}

pub fn warn_if_unprivileged() {
    if cfg!(windows) && !permissions::has_full_disk_access() {
        eprintln!(
            "note: not running as administrator; SystemCaches, WindowsUpdate and system crash \
             dumps/error reports are only partially accessible."
        );
    }
}
