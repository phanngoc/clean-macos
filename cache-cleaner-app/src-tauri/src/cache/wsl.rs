//! WSL 2 distributions (Windows only).
//!
//! Each WSL 2 distro lives in an `ext4.vhdx` on the Windows drive. Files deleted inside the
//! distro only give space back to Windows when the VHDX is sparse and the freed blocks are
//! discarded, so every clean ends with `fstrim /` in the distro.

use super::CacheType;
use crate::utils::process::command;
use anyhow::{anyhow, Result};
use std::io::Write;
use std::process::{Output, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Distro {
    pub name: String,
    pub running: bool,
    pub version: u8,
    pub is_default: bool,
}

/// `find` roots (shell words, globs allowed) and the predicate selecting deletable files.
struct Target {
    roots: &'static str,
    predicate: &'static str,
}

/// Package caches, archived journals, rotated logs, crash reports and stale temp files.
const SYSTEM_TARGETS: &[Target] = &[
    Target { roots: "/var/cache/apt/archives", predicate: "-type f -name '*.deb'" },
    // Archived journals only (`system@...journal~`); the active `system.journal` is kept.
    Target { roots: "/var/log/journal", predicate: "-type f -name '*@*.journal*' -mtime +7" },
    Target { roots: "/var/log", predicate: "-type f ( -name '*.gz' -o -name '*.[0-9]' -o -name '*.old' )" },
    Target { roots: "/tmp /var/tmp", predicate: "-type f -mtime +0" },
    Target { roots: "/var/crash", predicate: "-type f" },
];

/// Per-user caches of root and every /home user.
const USER_TARGETS: &[Target] = &[
    Target { roots: "/root/.cache /home/*/.cache", predicate: "-type f" },
    Target { roots: "/root/.npm/_cacache /home/*/.npm/_cacache", predicate: "-type f" },
];

fn targets(cache_type: &CacheType) -> Option<&'static [Target]> {
    match cache_type {
        CacheType::WslSystemCaches => Some(SYSTEM_TARGETS),
        CacheType::WslUserCaches => Some(USER_TARGETS),
        _ => None,
    }
}

pub fn is_wsl_type(cache_type: &CacheType) -> bool {
    targets(cache_type).is_some()
}

/// Shell script printing `<bytes> <files>` for the matched files, deleting them when `delete`.
fn find_script(targets: &[Target], delete: bool) -> String {
    let action = if delete { "-printf '%s\\n' -delete" } else { "-printf '%s\\n'" };
    let mut script = String::from("{\n");
    for target in targets {
        let predicate = target.predicate.replace("( ", "\\( ").replace(" )", " \\)");
        script.push_str(&format!(
            "for r in {}; do [ -d \"$r\" ] && find \"$r\" -xdev -mindepth 1 {} {}; done\n",
            target.roots, predicate, action
        ));
    }
    script.push_str("} 2>/dev/null | awk '{s+=$1; n++} END {print s+0, n+0}'\n");
    script
}

fn parse_totals(stdout: &str) -> (u64, usize) {
    let mut fields = stdout.split_whitespace();
    let bytes = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let files = fields.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    (bytes, files)
}

/// Totals over every cleanable distro: `(bytes, files, distro names)`.
pub fn scan(cache_type: &CacheType) -> Result<(u64, usize, Vec<String>)> {
    run_targets(cache_type, false)
}

/// Deletes the matched files in every cleanable distro, then trims so Windows gets the space back.
pub fn clean(cache_type: &CacheType) -> Result<(u64, usize, Vec<String>)> {
    let totals = run_targets(cache_type, true)?;
    for distro in &totals.2 {
        trim(distro);
    }
    Ok(totals)
}

fn run_targets(cache_type: &CacheType, delete: bool) -> Result<(u64, usize, Vec<String>)> {
    let targets = targets(cache_type).ok_or_else(|| anyhow!("{:?} is not a WSL cache", cache_type))?;
    let script = find_script(targets, delete);
    let distros = cleanable_distros();
    if distros.is_empty() {
        return Err(anyhow!("No WSL 2 distribution found"));
    }
    let (mut bytes, mut files) = (0, 0);
    for distro in &distros {
        let output = run_script(&distro.name, &script)?;
        let (b, f) = parse_totals(&String::from_utf8_lossy(&output.stdout));
        bytes += b;
        files += f;
    }
    Ok((bytes, files, distros.into_iter().map(|d| d.name).collect()))
}

/// Discards free blocks of the distro's root filesystem so a sparse VHDX shrinks on Windows.
pub fn trim(distro: &str) -> bool {
    run_script(distro, "fstrim /").map(|o| o.status.success()).unwrap_or(false)
}

/// Runs `script` with `sh` as root inside `distro` (no sudo password needed), script on stdin.
pub fn run_script(distro: &str, script: &str) -> Result<Output> {
    let mut child = command("wsl.exe")
        .args(["-d", distro, "-u", "root", "-e", "sh", "-s"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("wsl.exe stdin unavailable"))?
        .write_all(script.as_bytes())?;
    Ok(child.wait_with_output()?)
}

/// Installed distributions (`wsl.exe -l -v`). Empty when WSL is not installed.
pub fn distros() -> Vec<Distro> {
    if !cfg!(windows) {
        return Vec::new();
    }
    match command("wsl.exe").args(["-l", "-v"]).env("WSL_UTF8", "1").output() {
        Ok(output) if output.status.success() => parse_distro_list(&decode(&output.stdout)),
        _ => Vec::new(),
    }
}

/// WSL 2 distros whose filesystem we may clean (Docker Desktop manages its own distros).
pub fn cleanable_distros() -> Vec<Distro> {
    distros()
        .into_iter()
        .filter(|d| d.version == 2 && !d.name.starts_with("docker-desktop"))
        .collect()
}

pub fn default_distro() -> Option<Distro> {
    distros().into_iter().find(|d| d.is_default)
}

/// wsl.exe prints UTF-16LE unless WSL_UTF8 is honoured (older builds ignore it).
fn decode(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes.iter().skip(1).step_by(2).filter(|&&b| b == 0).count() > bytes.len() / 4 {
        let units: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn parse_distro_list(text: &str) -> Vec<Distro> {
    text.lines()
        .skip(1) // header: NAME STATE VERSION
        .filter_map(|line| {
            let line = line.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
            let (is_default, rest) = match line.strip_prefix('*') {
                Some(rest) => (true, rest),
                None => (false, line),
            };
            let mut cols = rest.split_whitespace();
            let name = cols.next()?.to_string();
            let state = cols.next()?;
            let version = cols.next()?.parse().ok()?;
            Some(Distro { name, running: state.eq_ignore_ascii_case("Running"), version, is_default })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_distro_list_marks_default_and_versions() {
        let text = "  NAME                   STATE           VERSION\r\n\
                    * Ubuntu-24.04           Running         2\r\n  \
                    docker-desktop         Stopped         2\r\n  \
                    Legacy                 Stopped         1\r\n";
        let distros = parse_distro_list(text);
        assert_eq!(distros.len(), 3);
        assert_eq!(
            distros[0],
            Distro { name: "Ubuntu-24.04".into(), running: true, version: 2, is_default: true }
        );
        assert!(!distros[1].is_default && !distros[1].running);
        assert_eq!(distros[2].version, 1);
    }

    #[test]
    fn test_decode_utf16_output() {
        let utf16: Vec<u8> = "NAME\r\n* Ubuntu 2".encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
        assert_eq!(decode(&utf16), "NAME\r\n* Ubuntu 2");
        assert_eq!(decode(b"NAME"), "NAME");
    }

    #[test]
    fn test_find_script_only_deletes_when_asked() {
        let dry = find_script(SYSTEM_TARGETS, false);
        assert!(!dry.contains("-delete"));
        assert!(dry.contains("\\( -name '*.gz'"));
        assert!(find_script(USER_TARGETS, true).contains("-delete"));
    }

    #[test]
    fn test_parse_totals() {
        assert_eq!(parse_totals("1234 5\n"), (1234, 5));
        assert_eq!(parse_totals(""), (0, 0));
    }

    #[cfg(windows)]
    #[test]
    fn test_find_script_runs_in_wsl_without_deleting() {
        // Dry-run script must execute cleanly in a real distro when one exists.
        let Some(distro) = cleanable_distros().into_iter().next() else { return };
        let output = run_script(&distro.name, &find_script(SYSTEM_TARGETS, false)).unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert_eq!(stdout.split_whitespace().count(), 2, "unexpected output: {stdout}");
    }
}
