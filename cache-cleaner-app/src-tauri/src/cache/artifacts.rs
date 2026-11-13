//! Rebuildable build output (`node_modules`, Rust `target`, `.next`, `.venv`, CACHEDIR.TAG dirs…)
//! of projects that have been idle for a while.
//!
//! A project is *active* — and left completely alone — when any file outside build output, or its
//! git index/HEAD, changed within the idle window. The project is the enclosing git repository
//! (so a monorepo counts as one project), else the directory holding the manifest.

use crate::utils::filesystem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use walkdir::WalkDir;

/// Build-output directory names and the manifest that must sit next to them.
const KINDS: &[(&str, &[&str])] = &[
    ("node_modules", &["package.json"]),
    (".next", &["package.json"]),
    (".nuxt", &["package.json"]),
    (".turbo", &["package.json"]),
    (".parcel-cache", &["package.json"]),
    (".svelte-kit", &["package.json"]),
    ("target", &["Cargo.toml"]),
    (".venv", &["pyproject.toml", "requirements.txt", "setup.py"]),
    (".gradle", &["build.gradle", "build.gradle.kts", "settings.gradle", "settings.gradle.kts"]),
];

/// <https://bford.info/cachedir/>: directories carrying this tag are caches by contract (cargo
/// writes it into every target dir, including shared `CARGO_TARGET_DIR`s).
const CACHEDIR_TAG_SIGNATURE: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55";
pub const CACHEDIR_KIND: &str = "CACHEDIR.TAG";
/// Kind of a partial clean of an *active* tagged cache dir (see `sweep`).
pub const SWEEP_KIND: &str = "sweep";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub path: PathBuf,
    pub kind: String,
    pub size_bytes: u64,
    /// Project whose idleness was checked.
    pub project: PathBuf,
    /// Sweep: only files not read for this long are removed; the directory stays.
    pub unread_for: Option<Duration>,
}

fn is_artifact_name(name: &str) -> bool {
    KINDS.iter().any(|(kind, _)| *kind == name)
}

fn artifact_kind(dir: &Path) -> Option<String> {
    let name = dir.file_name()?.to_string_lossy();
    if let Some((kind, markers)) = KINDS.iter().find(|(kind, _)| *kind == name) {
        let parent = dir.parent()?;
        if markers.iter().any(|m| parent.join(m).is_file()) {
            return Some(kind.to_string());
        }
    }
    is_cachedir_tagged(dir).then(|| CACHEDIR_KIND.to_string())
}

fn is_cachedir_tagged(dir: &Path) -> bool {
    let mut tag = [0u8; CACHEDIR_TAG_SIGNATURE.len()];
    std::fs::File::open(dir.join("CACHEDIR.TAG"))
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut tag))
        .is_ok_and(|()| tag == CACHEDIR_TAG_SIGNATURE)
}

/// Enclosing git repository below `scan_root`, else `dir`.
fn project_root(dir: &Path, scan_root: &Path) -> PathBuf {
    dir.ancestors()
        .take_while(|p| p.starts_with(scan_root))
        .find(|p| p.join(".git").exists())
        .unwrap_or(dir)
        .to_path_buf()
}

fn modified_after(path: &Path, cutoff: SystemTime) -> bool {
    std::fs::metadata(path).and_then(|m| m.modified()).is_ok_and(|t| t > cutoff)
}

/// Any source file (build output and `.git` excluded) or git index/HEAD newer than `cutoff`.
fn recently_modified(project: &Path, cutoff: SystemTime) -> bool {
    let git = project.join(".git");
    if ["index", "HEAD", "logs/HEAD"].iter().any(|f| modified_after(&git.join(f), cutoff)) {
        return true;
    }
    WalkDir::new(project)
        .into_iter()
        .filter_entry(|e| {
            e.depth() == 0 || {
                let name = e.file_name().to_string_lossy();
                name != ".git" && !is_artifact_name(&name)
            }
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .any(|e| e.metadata().ok().and_then(|m| m.modified().ok()).is_some_and(|t| t > cutoff))
}

/// Build output under `roots` belonging to projects idle for at least `idle`, largest first.
///
/// With `sweep`, active CACHEDIR-tagged dirs (e.g. a shared cargo target being built daily) are
/// reported too, sized by the files not *read* within `sweep` (cargo-sweep semantics: rebuilt on
/// demand). Needs access times; NTFS may not maintain them.
pub fn find_artifacts(roots: &[PathBuf], idle: Duration, sweep: Option<Duration>) -> Vec<Artifact> {
    let cutoff = SystemTime::now() - idle;
    let mut active: HashMap<PathBuf, bool> = HashMap::new();
    let mut found = Vec::new();

    for root in roots {
        let mut walker = WalkDir::new(root).into_iter();
        while let Some(entry) = walker.next() {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_dir() {
                continue;
            }
            if entry.file_name() == ".git" {
                walker.skip_current_dir();
                continue;
            }
            let Some(kind) = artifact_kind(entry.path()) else {
                // Hidden dirs (.vscode-server, .local, .nvm, ...) hold installed tools whose
                // node_modules look like projects; only build output itself is taken from them.
                if entry.depth() > 0 && entry.file_name().to_string_lossy().starts_with('.') {
                    walker.skip_current_dir();
                }
                continue;
            };
            walker.skip_current_dir();

            let dir = entry.path();
            let project = if kind == CACHEDIR_KIND {
                dir.to_path_buf()
            } else {
                project_root(dir.parent().unwrap_or(dir), root)
            };
            let is_active = *active
                .entry(project.clone())
                .or_insert_with(|| recently_modified(&project, cutoff));
            if is_active {
                // Tagged dirs only: cargo `target`, shared CARGO_TARGET_DIRs, tool caches.
                if let (true, Some(unread_for)) = (is_cachedir_tagged(dir), sweep) {
                    let size_bytes = unread_files(dir, unread_for).filter_map(|e| e.metadata().ok()).map(|m| m.len()).sum();
                    if size_bytes > 0 {
                        found.push(Artifact {
                            path: dir.to_path_buf(),
                            kind: SWEEP_KIND.to_string(),
                            size_bytes,
                            project,
                            unread_for: Some(unread_for),
                        });
                    }
                }
                continue;
            }
            found.push(Artifact {
                path: dir.to_path_buf(),
                kind,
                size_bytes: filesystem::calculate_dir_size_sync(dir).unwrap_or(0),
                project,
                unread_for: None,
            });
        }
    }
    found.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    found
}

/// Files below `dir` whose last access is older than `unread_for`.
fn unread_files(dir: &Path, unread_for: Duration) -> impl Iterator<Item = walkdir::DirEntry> {
    let cutoff = SystemTime::now() - unread_for;
    WalkDir::new(dir).into_iter().filter_map(|e| e.ok()).filter(move |e| {
        e.file_type().is_file()
            && e.metadata().ok().and_then(|m| m.accessed().ok()).is_some_and(|t| t < cutoff)
    })
}

/// Deletes the artifact directory entirely, or only its unread files for a sweep.
pub fn remove_artifact(artifact: &Artifact) -> filesystem::RemovalStats {
    if let Some(unread_for) = artifact.unread_for {
        let mut stats = filesystem::RemovalStats::default();
        for entry in unread_files(&artifact.path, unread_for) {
            let len = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if std::fs::remove_file(entry.path()).is_ok() {
                stats.freed_bytes += len;
                stats.removed += 1;
            } else {
                stats.failed += 1;
            }
        }
        return stats;
    }
    let mut stats = filesystem::remove_contents(&artifact.path, None);
    if std::fs::remove_dir(&artifact.path).is_ok() {
        stats.removed += 1;
    }
    stats
}

/// Same rules as [`find_artifacts`] as a POSIX shell script, for roots inside a WSL distro.
/// Prints `size<TAB>kind<TAB>path<TAB>project` per idle artifact (and per sweep when
/// `sweep_days > 0`); deletes them when `delete`.
pub fn wsl_script(roots: &[String], idle_days: u64, sweep_days: u64, delete: bool) -> String {
    let names: Vec<&str> = KINDS.iter().map(|(kind, _)| *kind).collect();
    let prune = names.iter().map(|n| format!("-name '{n}'")).collect::<Vec<_>>().join(" -o ");
    let quoted_roots = roots.iter().map(|r| format!("'{}'", r.replace('\'', "'\\''"))).collect::<Vec<_>>().join(" ");
    format!(
        r#"IDLE={idle_days}; SWEEP={sweep_days}; DELETE={delete}; SEEN='|'; ACTIVE='|'; IDLEP='|'
kind_of() {{
  p=$(dirname "$1"); n=$(basename "$1")
  case "$n" in
    node_modules|.next|.nuxt|.turbo|.parcel-cache|.svelte-kit) [ -f "$p/package.json" ] && {{ echo "$n"; return; }};;
    target) [ -f "$p/Cargo.toml" ] && {{ echo "$n"; return; }};;
    .venv) {{ [ -f "$p/pyproject.toml" ] || [ -f "$p/requirements.txt" ] || [ -f "$p/setup.py" ]; }} && {{ echo "$n"; return; }};;
    .gradle) ls "$p"/build.gradle* "$p"/settings.gradle* >/dev/null 2>&1 && {{ echo "$n"; return; }};;
  esac
  head -c 43 "$1/CACHEDIR.TAG" 2>/dev/null | grep -q '^Signature: 8a477f597d28d172789f06886806bc55' && echo {CACHEDIR_KIND}
}}
is_active() {{
  for f in index HEAD logs/HEAD; do [ -n "$(find "$1/.git/$f" -mtime -$IDLE 2>/dev/null)" ] && return 0; done
  [ -n "$(find "$1" -xdev -mindepth 1 \( -name .git -o {prune} \) -prune -o -type f -mtime -$IDLE -print -quit 2>/dev/null)" ]
}}
find {quoted_roots} -xdev -mindepth 1 -type d \( -name '.*' -o {prune} \) -prune -print -o -type f -name CACHEDIR.TAG -print 2>/dev/null |
while IFS= read -r c; do
  [ "$(basename "$c")" = CACHEDIR.TAG ] && [ -f "$c" ] && c=$(dirname "$c")
  [ "$(basename "$c")" = .git ] && continue
  case "$SEEN" in *"|$c|"*) continue;; esac
  for s in $(echo "$SEEN" | tr '|' ' '); do case "$c" in "$s"/*) continue 2;; esac; done
  k=$(kind_of "$c"); [ -n "$k" ] || continue
  SEEN="$SEEN$c|"
  if [ "$k" = {CACHEDIR_KIND} ]; then proj="$c"
  else proj=$(git -c safe.directory='*' -C "$(dirname "$c")" rev-parse --show-toplevel 2>/dev/null || dirname "$c"); fi
  case "$ACTIVE" in *"|$proj|"*) act=1;; *)
    case "$IDLEP" in *"|$proj|"*) act=0;; *)
      if is_active "$proj"; then act=1; ACTIVE="$ACTIVE$proj|"; else act=0; IDLEP="$IDLEP$proj|"; fi;;
    esac;;
  esac
  if [ $act = 1 ]; then
    if [ "$SWEEP" -gt 0 ] && head -c 43 "$c/CACHEDIR.TAG" 2>/dev/null | grep -q '^Signature: 8a477f597d28d172789f06886806bc55'; then
      s=$(find "$c" -xdev -type f -atime +$SWEEP -printf '%s\n' 2>/dev/null | awk '{{t+=$1}} END {{print t+0}}')
      [ "$s" -gt 0 ] && printf '%s\t%s\t%s\t%s\n' "$s" {SWEEP_KIND} "$c" "$proj"
      [ "$DELETE" = true ] && find "$c" -xdev -type f -atime +$SWEEP -delete 2>/dev/null
    fi
    continue
  fi
  printf '%s\t%s\t%s\t%s\n' "$(du -sxb "$c" | cut -f1)" "$k" "$c" "$proj"
  [ "$DELETE" = true ] && rm -rf -- "$c"
done
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn touch_old(path: &Path) {
        let old = SystemTime::now() - Duration::from_secs(90 * 24 * 3600);
        fs::File::options().write(true).open(path).unwrap().set_modified(old).unwrap();
    }

    fn project(root: &Path, name: &str, recent: bool) -> PathBuf {
        let dir = root.join(name);
        fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        fs::write(dir.join("package.json"), "{}").unwrap();
        fs::write(dir.join("node_modules/pkg/index.js"), "x".repeat(100)).unwrap();
        fs::write(dir.join("index.js"), "x").unwrap();
        touch_old(&dir.join("package.json"));
        if !recent {
            touch_old(&dir.join("index.js"));
        }
        dir
    }

    #[test]
    fn test_only_idle_projects_are_reported() {
        let root = tempfile::tempdir().unwrap();
        let idle = project(root.path(), "idle", false);
        project(root.path(), "active", true);

        let found = find_artifacts(&[root.path().to_path_buf()], Duration::from_secs(30 * 24 * 3600), None);

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].path, idle.join("node_modules"));
        assert_eq!(found[0].kind, "node_modules");
        assert_eq!(found[0].size_bytes, 100);
    }

    #[test]
    fn test_monorepo_activity_protects_nested_packages() {
        let root = tempfile::tempdir().unwrap();
        let repo = root.path().join("repo");
        fs::create_dir_all(repo.join(".git")).unwrap();
        // Nested package untouched for months, but another file in the repo changed today.
        project(&repo, "packages/old", false);
        fs::write(repo.join("README.md"), "today").unwrap();

        let found = find_artifacts(&[root.path().to_path_buf()], Duration::from_secs(30 * 24 * 3600), None);
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn test_installed_tools_in_hidden_dirs_are_not_projects() {
        let root = tempfile::tempdir().unwrap();
        // e.g. ~/.vscode-server/bin/<hash>/node_modules: the editor's runtime, not build output.
        project(&root.path().join(".vscode-server/bin"), "abc123", false);
        let found = find_artifacts(&[root.path().to_path_buf()], Duration::from_secs(24 * 3600), None);
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn test_target_requires_manifest() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("docs/target")).unwrap();
        assert_eq!(artifact_kind(&root.path().join("docs/target")), None);
        fs::write(root.path().join("docs/Cargo.toml"), "").unwrap();
        assert_eq!(artifact_kind(&root.path().join("docs/target")).as_deref(), Some("target"));
    }

    #[test]
    fn test_cachedir_tag_detection() {
        let root = tempfile::tempdir().unwrap();
        let shared = root.path().join("shared-target");
        fs::create_dir(&shared).unwrap();
        fs::write(shared.join("CACHEDIR.TAG"), "Signature: 8a477f597d28d172789f06886806bc55\n# cargo").unwrap();
        assert_eq!(artifact_kind(&shared).as_deref(), Some(CACHEDIR_KIND));
        fs::write(shared.join("CACHEDIR.TAG"), "Signature: nope").unwrap();
        assert_eq!(artifact_kind(&shared), None);
    }

    #[test]
    fn test_remove_artifact_deletes_directory() {
        let root = tempfile::tempdir().unwrap();
        let idle = project(root.path(), "idle", false);
        let found = find_artifacts(&[root.path().to_path_buf()], Duration::from_secs(24 * 3600), None);
        let stats = remove_artifact(&found[0]);
        assert_eq!(stats.failed, 0);
        assert!(!idle.join("node_modules").exists());
        assert!(idle.join("package.json").exists());
    }

    #[test]
    fn test_sweep_removes_only_unread_files_of_active_cache_dir() {
        let root = tempfile::tempdir().unwrap();
        let shared = root.path().join("shared-target");
        fs::create_dir_all(shared.join("debug")).unwrap();
        fs::write(shared.join("CACHEDIR.TAG"), "Signature: 8a477f597d28d172789f06886806bc55").unwrap();
        fs::write(shared.join("debug/fresh.rlib"), "fresh").unwrap(); // makes the dir active
        let stale = shared.join("debug/stale.rlib");
        fs::write(&stale, "0123456789").unwrap();
        let old = SystemTime::now() - Duration::from_secs(30 * 24 * 3600);
        let times = fs::FileTimes::new().set_accessed(old).set_modified(old);
        fs::File::options().write(true).open(&stale).unwrap().set_times(times).unwrap();

        let month = Duration::from_secs(30 * 24 * 3600);
        let week = Duration::from_secs(7 * 24 * 3600);
        assert!(find_artifacts(&[root.path().to_path_buf()], month, None).is_empty());
        let found = find_artifacts(&[root.path().to_path_buf()], month, Some(week));

        assert_eq!(found.len(), 1);
        assert_eq!((found[0].kind.as_str(), found[0].size_bytes), (SWEEP_KIND, 10));
        remove_artifact(&found[0]);
        assert!(!stale.exists());
        assert!(shared.join("debug/fresh.rlib").exists());
    }

    #[cfg(windows)]
    #[test]
    fn test_wsl_script_matches_rules() {
        // Fixture inside the distro: idle + active npm projects, and an active tagged cache dir
        // holding one file unread for a month.
        let Some(distro) = crate::cache::wsl::cleanable_distros().into_iter().next() else { return };
        let setup = "set -e; R=$(mktemp -d); for p in idle active; do mkdir -p $R/$p/node_modules/x; \
            echo '{}' > $R/$p/package.json; echo x > $R/$p/node_modules/x/a.js; echo x > $R/$p/i.js; done; \
            touch -d '90 days ago' $R/idle/i.js $R/idle/package.json $R/active/package.json; \
            mkdir -p $R/.tool/node_modules/x; echo '{}' > $R/.tool/package.json; \
            touch -d '90 days ago' $R/.tool/package.json $R/.tool/node_modules/x; \
            mkdir -p $R/tgt/debug; echo 'Signature: 8a477f597d28d172789f06886806bc55' > $R/tgt/CACHEDIR.TAG; \
            echo new > $R/tgt/debug/new; echo old > $R/tgt/debug/old; touch -a -d '30 days ago' $R/tgt/debug/old; echo $R";
        let run = |script: &str| {
            let out = crate::cache::wsl::run_script(&distro.name, script).unwrap();
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        };
        let dir = run(setup);
        let listed = run(&wsl_script(&[dir.clone()], 30, 7, true));
        let remaining = run(&format!("cd '{dir}' && find . -type f | sort; rm -rf '{dir}'"));

        let lines: Vec<&str> = listed.lines().collect();
        assert_eq!(lines.len(), 2, "{listed}");
        assert!(lines.iter().any(|l| l.ends_with(&format!("{SWEEP_KIND}\t{dir}/tgt\t{dir}/tgt"))), "{listed}");
        assert!(lines.iter().any(|l| l.contains(&format!("node_modules\t{dir}/idle/node_modules"))), "{listed}");
        assert!(!remaining.contains("./idle/node_modules"), "{remaining}");
        assert!(remaining.contains("./active/node_modules/x/a.js"), "{remaining}");
        assert!(remaining.contains("./tgt/debug/new") && !remaining.contains("./tgt/debug/old"), "{remaining}");
    }
}
