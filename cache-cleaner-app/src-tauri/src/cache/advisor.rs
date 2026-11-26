//! Cleanup suggestions for the `suggest` CLI: gathers candidates from every source (cache
//! catalog incl. Docker/WSL, build output of idle projects on Windows and in WSL, large stale
//! folders, custom scanners), tags how risky deleting each one is, and cleans a selection.
//!
//! What to pre-select is decided by [`super::memory`] from the user's history.

use super::artifacts::{self, Artifact, WSL_PREFIX};
use super::config::AppConfig;
use super::memory::{Memory, QUICK_WINDOW};
use super::registry::ScannerRegistry;
use super::scanner_trait::CacheScanner;
use super::{cleaner, paths, scanner, smart_suggestions, wsl, CacheInfo, CacheType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DAY: Duration = Duration::from_secs(24 * 3600);

/// Folders under the home directory searched for project build output by default.
const DEV_DIRS: &[&str] = &[
    "Documents", "Desktop", "projects", "code", "src", "dev", "workspace", "work", "repos",
    "source/repos", "git", "github", "Developer",
];
/// Searched inside the primary WSL distro by default.
const WSL_DEV_ROOTS: &[&str] = &["/home"];

/// How much deleting a suggestion can hurt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    /// Rebuilt or re-downloaded on demand.
    Safe,
    /// Usually fine, but may hold things that are slow or impossible to get back.
    Review,
    /// Holds user data (volumes, device backups, archives). Never pre-selected.
    Danger,
}

/// What a suggestion points at; stable across runs, stored in memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Target {
    Cache { cache_type: CacheType },
    /// `path` is prefixed with `wsl:` for build output inside WSL.
    Artifact { path: String, artifact_kind: String, project: String },
    Folder { path: String },
    Custom { id: String },
}

impl Target {
    pub fn id(&self) -> String {
        match self {
            Target::Cache { cache_type } => format!("cache:{cache_type:?}"),
            Target::Artifact { path, .. } => format!("artifact:{path}"),
            Target::Folder { path } => format!("folder:{path}"),
            Target::Custom { id } => format!("custom:{id}"),
        }
    }

    pub fn risk(&self) -> Risk {
        match self {
            Target::Cache { cache_type } => cache_risk(cache_type).0,
            Target::Artifact { artifact_kind, .. } => artifact_risk(artifact_kind).0,
            Target::Folder { .. } | Target::Custom { .. } => Risk::Review,
        }
    }
}

/// How to clean a candidate (not serialized).
#[derive(Debug, Clone)]
pub enum Action {
    Cache(CacheType),
    Artifact(Artifact),
    Folder(PathBuf),
    Custom(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub id: String,
    pub target: Target,
    pub label: String,
    pub location: String,
    pub size_bytes: u64,
    pub risk: Risk,
    /// What deleting it means.
    pub note: String,
    /// Id of a cache suggestion whose clean already deletes this one.
    pub covered_by: Option<String>,
    #[serde(skip)]
    pub action: Action,
    /// Local directories this candidate empties completely (to detect nested candidates).
    #[serde(skip)]
    owned_dirs: Vec<PathBuf>,
}

impl Candidate {
    fn new(target: Target, label: String, location: String, size_bytes: u64, note: String, action: Action) -> Self {
        Self {
            id: target.id(),
            risk: target.risk(),
            target,
            label,
            location,
            size_bytes,
            note,
            covered_by: None,
            action,
            owned_dirs: Vec::new(),
        }
    }

    fn local_path(&self) -> Option<&Path> {
        match &self.action {
            Action::Artifact(a) if !is_wsl_path(&a.path) => Some(&a.path),
            Action::Folder(p) => Some(p),
            _ => None,
        }
    }
}

fn is_wsl_path(path: &Path) -> bool {
    path.to_str().is_some_and(|p| p.starts_with(WSL_PREFIX))
}

/// Native separators (`C:\a/b` → `C:\a\b`) so the same folder always gets the same id.
fn normalize(path: &Path) -> PathBuf {
    path.components().collect()
}

/// `child` is `parent` or below it. Case-insensitive on Windows, where `uv\cache` and
/// `uv\Cache` are the same directory.
fn path_within(child: &Path, parent: &Path) -> bool {
    if cfg!(windows) {
        let lower = |p: &Path| normalize(p).to_string_lossy().to_lowercase();
        Path::new(&lower(child)).starts_with(lower(parent))
    } else {
        child.starts_with(parent)
    }
}

pub fn cache_label(cache_type: &CacheType) -> &'static str {
    use CacheType::*;
    match cache_type {
        Npm => "npm cache",
        Chrome => "Chrome cache",
        CacheDir => "~/.cache",
        VSCode => "VS Code cache",
        Cursor => "Cursor cache",
        Safari => "Safari cache",
        Firefox => "Firefox cache",
        Arc => "Arc cache",
        Yarn => "Yarn cache",
        Pnpm => "pnpm store",
        Pip => "pip cache",
        CocoaPods => "CocoaPods cache",
        Gradle => "Gradle caches",
        Cargo => "Cargo registry cache",
        XcodeDerivedData => "Xcode DerivedData",
        XcodeArchives => "Xcode archives",
        XcodeSimulators => "Xcode simulators",
        SystemCaches => "System temp/caches",
        UserLogs => "User logs",
        TempFiles => "Temp files",
        IosBackups => "iOS backups",
        NuGet => "NuGet packages",
        GoBuild => "Go build cache",
        Uv => "uv cache",
        Edge => "Edge cache",
        WindowsUpdate => "Windows Update downloads",
        ThumbnailCache => "Thumbnail cache",
        ErrorReports => "Error reports",
        CrashDumps => "Crash dumps",
        ShaderCache => "GPU shader cache",
        RecycleBin => "Recycle Bin",
        WslSystemCaches => "WSL system caches",
        WslUserCaches => "WSL user caches",
        DockerContainers => "Docker stopped containers",
        DockerImages => "Docker unused images",
        DockerVolumes => "Docker unused volumes",
        DockerBuildCache => "Docker build cache",
    }
}

fn cache_risk(cache_type: &CacheType) -> (Risk, &'static str) {
    use CacheType::*;
    match cache_type {
        DockerVolumes => (Risk::Danger, "deletes the data of volumes no container uses"),
        IosBackups => (Risk::Danger, "deletes device backups"),
        XcodeArchives => (Risk::Danger, "deletes archived builds (needed to symbolicate crashes)"),
        RecycleBin => (Risk::Review, "permanently deletes what is in the bin"),
        DockerImages => (Risk::Review, "unused images must be pulled or built again"),
        DockerContainers => (Risk::Review, "stopped containers and their changes are removed"),
        XcodeSimulators => (Risk::Review, "simulator apps and data are removed"),
        WslUserCaches => (Risk::Review, "may hold downloaded models or browsers"),
        CacheDir => (Risk::Review, "may hold downloaded models or tool runtimes (slow to re-download)"),
        _ => (Risk::Safe, "rebuilt or re-downloaded on demand"),
    }
}

fn artifact_risk(kind: &str) -> (Risk, &'static str) {
    match kind {
        ".venv" => (Risk::Review, "recreate the virtualenv before running the project"),
        artifacts::SWEEP_KIND => (Risk::Review, "files not read recently; cargo rebuilds them"),
        _ => (Risk::Safe, "reinstalled/rebuilt when you work on the project again"),
    }
}

// --- Scan plans -------------------------------------------------------------------------------

/// Which folders `collect` looks at for large stale folders.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum FolderScan {
    #[default]
    None,
    /// Every folder under the smart-suggestion roots.
    All,
    /// Only these (re-measured; must still be direct children of a suggestion root).
    Only(Vec<PathBuf>),
}

#[derive(Debug, Clone, Default)]
pub struct ScanPlan {
    pub cache_types: Vec<CacheType>,
    pub local_roots: Vec<PathBuf>,
    pub wsl_roots: Vec<String>,
    pub folders: FolderScan,
    pub custom: bool,
    pub idle_days: u64,
    pub sweep_days: u64,
    pub min_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct PlanOptions {
    pub idle_days: u64,
    pub sweep_days: u64,
    pub min_bytes: u64,
    /// Extra roots for build output; `wsl:/path` for WSL.
    pub extra_roots: Vec<String>,
    pub artifacts: bool,
    pub wsl: bool,
}

impl ScanPlan {
    fn empty(opts: &PlanOptions) -> Self {
        Self { idle_days: opts.idle_days, sweep_days: opts.sweep_days, min_bytes: opts.min_bytes, ..Default::default() }
    }

    fn add_root(&mut self, root: &str) {
        match root.strip_prefix(WSL_PREFIX) {
            Some(wsl_root) => self.wsl_roots.push(wsl_root.to_string()),
            None => self.local_roots.push(PathBuf::from(root)),
        }
    }

    fn add_target(&mut self, target: &Target) {
        match target {
            Target::Cache { cache_type } => self.cache_types.push(cache_type.clone()),
            Target::Artifact { path, project, .. } => {
                if path.starts_with(WSL_PREFIX) {
                    self.wsl_roots.push(project.clone());
                } else {
                    self.local_roots.push(PathBuf::from(project));
                }
            }
            Target::Folder { path } => match &mut self.folders {
                FolderScan::Only(list) => list.push(PathBuf::from(path)),
                FolderScan::None => self.folders = FolderScan::Only(vec![PathBuf::from(path)]),
                FolderScan::All => {}
            },
            Target::Custom { .. } => self.custom = true,
        }
    }

    fn finish(mut self, opts: &PlanOptions) -> Self {
        if !opts.wsl {
            self.cache_types.retain(|t| !wsl::is_wsl_type(t));
            self.wsl_roots.clear();
        }
        if !self.wsl_roots.is_empty() && wsl::primary_distro().is_none() {
            self.wsl_roots.clear();
        }
        let mut seen = HashSet::new();
        self.cache_types.retain(|t| paths::is_supported(t) && seen.insert(format!("{t:?}")));
        self.local_roots = dedupe_roots(self.local_roots.into_iter().filter(|r| r.is_dir()).collect());
        self.wsl_roots = dedupe_roots(self.wsl_roots.into_iter().map(PathBuf::from).collect())
            .into_iter()
            .filter_map(|p| p.to_str().map(|s| s.replace('\\', "/")))
            .collect();
        self
    }

    pub fn is_empty(&self) -> bool {
        self.cache_types.is_empty()
            && self.local_roots.is_empty()
            && self.wsl_roots.is_empty()
            && self.folders == FolderScan::None
            && !self.custom
    }
}

/// Roots with any root nested in another one dropped (the outer walk covers it).
pub fn dedupe_roots(mut roots: Vec<PathBuf>) -> Vec<PathBuf> {
    roots.sort_by_key(|r| r.components().count());
    let mut kept: Vec<PathBuf> = Vec::new();
    for root in roots {
        if !kept.iter().any(|k| root.starts_with(k)) {
            kept.push(root);
        }
    }
    kept
}

pub fn default_dev_roots() -> Vec<PathBuf> {
    let Some(home) = dirs::home_dir() else { return Vec::new() };
    DEV_DIRS.iter().map(|d| home.join(d)).filter(|p| p.is_dir()).collect()
}

/// Everything: whole cache catalog, default dev roots (+ projects remembered from earlier runs),
/// large stale folders and custom scanners.
pub fn full_plan(opts: &PlanOptions, memory: &Memory) -> ScanPlan {
    let mut plan = ScanPlan::empty(opts);
    plan.cache_types = paths::supported_types().to_vec();
    plan.folders = FolderScan::All;
    plan.custom = true;
    if opts.artifacts {
        plan.local_roots = default_dev_roots();
        if cfg!(windows) {
            plan.wsl_roots = WSL_DEV_ROOTS.iter().map(|r| r.to_string()).collect();
        }
        for entry in memory.entries.values() {
            if matches!(entry.target, Target::Artifact { .. }) {
                plan.add_target(&entry.target);
            }
        }
    }
    for root in &opts.extra_roots {
        plan.add_root(root);
    }
    plan.finish(opts)
}

/// Only the places remembered from runs in the last [`QUICK_WINDOW`].
pub fn quick_plan(opts: &PlanOptions, memory: &Memory, now: u64) -> ScanPlan {
    let mut plan = ScanPlan::empty(opts);
    plan.custom = true;
    for (_, entry) in memory.recent(now, QUICK_WINDOW) {
        if opts.artifacts || !matches!(entry.target, Target::Artifact { .. }) {
            plan.add_target(&entry.target);
        }
    }
    for root in &opts.extra_roots {
        plan.add_root(root);
    }
    plan.finish(opts)
}

/// Plan re-measuring just `ids`. Ids must be remembered from an earlier scan, except
/// `cache:<Type>` which is always resolvable. Returns the unknown ids as error.
pub fn plan_for_ids(ids: &[String], opts: &PlanOptions, memory: &Memory) -> Result<ScanPlan, Vec<String>> {
    let mut plan = ScanPlan::empty(opts);
    let mut unknown = Vec::new();
    for id in ids {
        if let Some(entry) = memory.entries.get(id) {
            plan.add_target(&entry.target);
        } else if let Some(cache_type) = id.strip_prefix("cache:").and_then(|t| CacheType::from_str(t).ok()) {
            plan.add_target(&Target::Cache { cache_type });
        } else {
            unknown.push(id.clone());
        }
    }
    let wants_wsl = ids.iter().any(|id| id.starts_with("artifact:wsl:") || id.starts_with("cache:Wsl"));
    let opts = PlanOptions { wsl: opts.wsl || wants_wsl, ..opts.clone() };
    if unknown.is_empty() { Ok(plan.finish(&opts)) } else { Err(unknown) }
}

// --- Collect ----------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Caches,
    Artifacts,
    WslArtifacts,
    Folders,
    Custom,
}

impl Source {
    pub const ALL: [Source; 5] = [Source::Caches, Source::Artifacts, Source::WslArtifacts, Source::Folders, Source::Custom];

    pub fn name(self) -> &'static str {
        match self {
            Source::Caches => "caches, Docker & WSL",
            Source::Artifacts => "build output",
            Source::WslArtifacts => "build output in WSL",
            Source::Folders => "large stale folders",
            Source::Custom => "custom scanners",
        }
    }

    pub fn enabled(self, plan: &ScanPlan) -> bool {
        match self {
            Source::Caches => !plan.cache_types.is_empty(),
            Source::Artifacts => !plan.local_roots.is_empty(),
            Source::WslArtifacts => !plan.wsl_roots.is_empty(),
            Source::Folders => plan.folders != FolderScan::None,
            Source::Custom => plan.custom,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SourceState {
    Started,
    Done { found: usize, bytes: u64, elapsed: Duration },
    Failed(String),
}

/// Runs every enabled source of `plan` concurrently; `on` hears about each source's progress.
pub async fn collect(plan: &ScanPlan, on: &(dyn Fn(Source, SourceState) + Sync)) -> Vec<Candidate> {
    let sweep = (plan.sweep_days > 0).then(|| DAY * plan.sweep_days as u32);
    let idle = DAY * plan.idle_days.max(1) as u32;

    let (caches, local, in_wsl, folders, custom) = tokio::join!(
        tracked(Source::Caches, plan, on, scan_caches(&plan.cache_types)),
        tracked(Source::Artifacts, plan, on, scan_local_artifacts(plan.local_roots.clone(), idle, sweep)),
        tracked(Source::WslArtifacts, plan, on, scan_wsl_artifacts(plan.wsl_roots.clone(), plan.idle_days, plan.sweep_days)),
        tracked(Source::Folders, plan, on, scan_folders(&plan.folders, plan.min_bytes)),
        tracked(Source::Custom, plan, on, scan_custom()),
    );
    finalize([caches, local, in_wsl, folders, custom].concat())
}

async fn tracked(
    source: Source,
    plan: &ScanPlan,
    on: &(dyn Fn(Source, SourceState) + Sync),
    scan: impl Future<Output = Result<Vec<Candidate>>>,
) -> Vec<Candidate> {
    if !source.enabled(plan) {
        return Vec::new();
    }
    on(source, SourceState::Started);
    let started = Instant::now();
    match scan.await {
        Ok(found) => {
            let bytes = found.iter().map(|c| c.size_bytes).sum();
            on(source, SourceState::Done { found: found.len(), bytes, elapsed: started.elapsed() });
            found
        }
        Err(err) => {
            on(source, SourceState::Failed(err.to_string()));
            Vec::new()
        }
    }
}

async fn scan_caches(types: &[CacheType]) -> Result<Vec<Candidate>> {
    let infos = futures::future::join_all(types.iter().map(scanner::scan_cache)).await;
    Ok(infos
        .into_iter()
        .filter_map(Result::ok)
        .filter(|info| info.exists && info.size > 0)
        .map(cache_candidate)
        .collect())
}

fn cache_candidate(info: CacheInfo) -> Candidate {
    let mut lines = info.path.lines();
    let first = lines.next().unwrap_or_default().to_string();
    let more = lines.count();
    let location = if more > 0 { format!("{first} (+{more} more)") } else { first };
    let (_, note) = cache_risk(&info.cache_type);
    // Fully emptied directories only: aged locations (temp) keep recent entries.
    let owned_dirs = paths::locations(&info.cache_type)
        .into_iter()
        .filter(|l| l.min_age.is_none())
        .map(|l| l.path)
        .collect();
    let target = Target::Cache { cache_type: info.cache_type.clone() };
    let mut candidate = Candidate::new(
        target,
        cache_label(&info.cache_type).to_string(),
        location,
        info.size,
        note.to_string(),
        Action::Cache(info.cache_type),
    );
    candidate.owned_dirs = owned_dirs;
    candidate
}

fn artifact_candidate(artifact: Artifact) -> Candidate {
    let path = artifact.path.to_string_lossy().into_owned();
    let project = artifact.project.to_string_lossy().into_owned();
    let project_name = project.rsplit(['/', '\\']).find(|s| !s.is_empty()).unwrap_or(&project).to_string();
    let (_, note) = artifact_risk(&artifact.kind);
    let where_ = if is_wsl_path(&artifact.path) { " (WSL)" } else { "" };
    let target = Target::Artifact { path: path.clone(), artifact_kind: artifact.kind.clone(), project };
    Candidate::new(
        target,
        format!("{} · {project_name}{where_}", artifact.kind),
        path,
        artifact.size_bytes,
        note.to_string(),
        Action::Artifact(artifact),
    )
}

async fn scan_local_artifacts(roots: Vec<PathBuf>, idle: Duration, sweep: Option<Duration>) -> Result<Vec<Candidate>> {
    let found = tokio::task::spawn_blocking(move || artifacts::find_artifacts(&roots, idle, sweep)).await?;
    Ok(found.into_iter().map(artifact_candidate).collect())
}

async fn scan_wsl_artifacts(roots: Vec<String>, idle_days: u64, sweep_days: u64) -> Result<Vec<Candidate>> {
    let (_, found) = tokio::task::spawn_blocking(move || {
        artifacts::find_wsl_artifacts(&roots, idle_days.max(1), sweep_days, false)
    })
    .await??;
    Ok(found.into_iter().map(artifact_candidate).collect())
}

fn folder_candidate(s: smart_suggestions::FolderSuggestion) -> Candidate {
    let mut note = "the whole folder is removed".to_string();
    if !s.reasons.is_empty() {
        note = format!("{note}; {}", s.reasons.join("; "));
    }
    let path = normalize(Path::new(&s.path));
    let shown = path.to_string_lossy().into_owned();
    Candidate::new(Target::Folder { path: shown.clone() }, s.name, shown, s.size_bytes, note, Action::Folder(path))
}

/// A remembered folder is only re-offered while it is still a direct child of a suggestion
/// root: memory is never trusted to point anywhere else.
fn is_suggestable_folder(path: &Path, roots: &[PathBuf]) -> bool {
    path.is_dir() && path.parent().is_some_and(|parent| roots.iter().any(|r| r == parent))
}

async fn scan_folders(scan: &FolderScan, min_bytes: u64) -> Result<Vec<Candidate>> {
    let found = match scan {
        FolderScan::None => Vec::new(),
        FolderScan::All => {
            let min_mb = (min_bytes / (1024 * 1024)).max(1);
            smart_suggestions::scan_suggestions(min_mb, 30).await?
        }
        FolderScan::Only(list) => {
            let roots = smart_suggestions::suggestion_roots();
            let mut found = Vec::new();
            for path in list.iter().filter(|p| is_suggestable_folder(p, &roots)) {
                if let Ok(info) = smart_suggestions::get_folder_info(&path.to_string_lossy()).await {
                    found.push(info);
                }
            }
            found
        }
    };
    Ok(found.into_iter().map(folder_candidate).collect())
}

async fn custom_registry() -> ScannerRegistry {
    let registry = ScannerRegistry::new();
    if let Ok(config) = AppConfig::load() {
        for scanner in config.custom_scanners {
            let _ = registry.register(scanner).await;
        }
    }
    registry
}

async fn scan_custom() -> Result<Vec<Candidate>> {
    let registry = custom_registry().await;
    let mut found = Vec::new();
    for scanner in registry.get_all().await {
        if let Ok(result) = scanner.scan().await {
            if result.exists && result.size_bytes > 0 {
                let path = result.path.to_string_lossy().into_owned();
                let mut candidate = Candidate::new(
                    Target::Custom { id: result.id.clone() },
                    result.name,
                    path,
                    result.size_bytes,
                    "your custom scanner".to_string(),
                    Action::Custom(result.id),
                );
                candidate.owned_dirs = vec![result.path];
                found.push(candidate);
            }
        }
    }
    Ok(found)
}

/// De-duplicates by id (largest wins), marks candidates nested in a fully-emptied cache
/// directory, sorts largest first.
pub fn finalize(candidates: Vec<Candidate>) -> Vec<Candidate> {
    let mut by_id: HashMap<String, Candidate> = HashMap::new();
    for candidate in candidates {
        match by_id.get(&candidate.id) {
            Some(existing) if existing.size_bytes >= candidate.size_bytes => {}
            _ => {
                by_id.insert(candidate.id.clone(), candidate);
            }
        }
    }
    let owners: Vec<(String, Vec<PathBuf>)> = by_id
        .values()
        .filter(|c| !c.owned_dirs.is_empty())
        .map(|c| (c.id.clone(), c.owned_dirs.clone()))
        .collect();
    let mut list: Vec<Candidate> = by_id.into_values().collect();
    for candidate in &mut list {
        let Some(path) = candidate.local_path().map(Path::to_path_buf) else { continue };
        candidate.covered_by = owners
            .iter()
            .find(|(id, dirs)| *id != candidate.id && dirs.iter().any(|d| path_within(&path, d)))
            .map(|(id, _)| id.clone());
    }
    list.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes).then_with(|| a.id.cmp(&b.id)));
    list
}

// --- Clean ------------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct Outcome {
    pub id: String,
    pub freed_bytes: u64,
    pub ok: bool,
    pub message: String,
}

/// Bytes the selection frees, not counting candidates already covered by another selected one.
pub fn selection_bytes(selected: &[&Candidate]) -> u64 {
    let ids: HashSet<&str> = selected.iter().map(|c| c.id.as_str()).collect();
    selected
        .iter()
        .filter(|c| !c.covered_by.as_deref().is_some_and(|p| ids.contains(p)))
        .map(|c| c.size_bytes)
        .sum()
}

/// Cleans `selected` one by one (`on_done` after each), then trims WSL distros that lost files.
pub async fn clean_selected(
    selected: &[&Candidate],
    dry_run: bool,
    mut on_done: impl FnMut(&Candidate, &Outcome),
) -> Vec<Outcome> {
    let ids: HashSet<&str> = selected.iter().map(|c| c.id.as_str()).collect();
    let mut outcomes = Vec::new();
    let mut trim_wsl = false;
    for candidate in selected {
        let outcome = match candidate.covered_by.as_deref().filter(|p| ids.contains(p)) {
            Some(parent) => Outcome {
                id: candidate.id.clone(),
                freed_bytes: 0,
                ok: true,
                message: format!("included in {parent}"),
            },
            None => clean_one(candidate, dry_run).await,
        };
        if !dry_run && outcome.ok {
            trim_wsl |= matches!(&candidate.action, Action::Artifact(a) if is_wsl_path(&a.path));
        }
        on_done(candidate, &outcome);
        outcomes.push(outcome);
    }
    if trim_wsl {
        if let Some(distro) = wsl::primary_distro() {
            let _ = tokio::task::spawn_blocking(move || wsl::trim(&distro.name)).await;
        }
    }
    outcomes
}

async fn clean_one(candidate: &Candidate, dry_run: bool) -> Outcome {
    let outcome = |freed_bytes, ok, message: String| Outcome { id: candidate.id.clone(), freed_bytes, ok, message };
    let would = |bytes| outcome(bytes, true, format!("would free {bytes} bytes"));
    match &candidate.action {
        Action::Cache(cache_type) => match cleaner::clean(cache_type, dry_run).await {
            Ok(r) => outcome(r.freed_bytes, r.success, r.message),
            Err(e) => outcome(0, false, e.to_string()),
        },
        Action::Artifact(_) | Action::Folder(_) | Action::Custom(_) if dry_run => would(candidate.size_bytes),
        Action::Artifact(artifact) if is_wsl_path(&artifact.path) => {
            let Some(script) = artifacts::wsl_remove_script(artifact) else {
                return outcome(0, false, "not a WSL path".to_string());
            };
            let Some(distro) = wsl::primary_distro() else {
                return outcome(0, false, "no WSL 2 distribution found".to_string());
            };
            match tokio::task::spawn_blocking(move || wsl::run_script(&distro.name, &script)).await {
                Ok(Ok(out)) if out.status.success() => outcome(candidate.size_bytes, true, "removed in WSL".to_string()),
                Ok(Ok(out)) => outcome(0, false, String::from_utf8_lossy(&out.stderr).trim().to_string()),
                Ok(Err(e)) => outcome(0, false, e.to_string()),
                Err(e) => outcome(0, false, e.to_string()),
            }
        }
        Action::Artifact(artifact) => {
            let artifact = artifact.clone();
            match tokio::task::spawn_blocking(move || artifacts::remove_artifact(&artifact)).await {
                Ok(stats) => {
                    let ok = stats.failed == 0 || stats.removed > 0;
                    let mut message = format!("removed {} entries", stats.removed);
                    if stats.failed > 0 {
                        message.push_str(&format!(", {} in use/locked", stats.failed));
                    }
                    outcome(stats.freed_bytes, ok, message)
                }
                Err(e) => outcome(0, false, e.to_string()),
            }
        }
        Action::Folder(path) => {
            if !path.exists() {
                return outcome(0, true, "already gone".to_string());
            }
            match smart_suggestions::remove_suggested_folders(vec![path.to_string_lossy().into_owned()]).await {
                Ok(r) => outcome(r.total_freed_bytes, r.success, r.message),
                Err(e) => outcome(0, false, e.to_string()),
            }
        }
        Action::Custom(id) => match custom_registry().await.clean_custom(id, false).await {
            Ok(r) => outcome(r.freed_bytes, r.success, r.message),
            Err(e) => outcome(0, false, e.to_string()),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts() -> PlanOptions {
        PlanOptions { idle_days: 30, sweep_days: 0, min_bytes: 0, extra_roots: vec![], artifacts: true, wsl: false }
    }

    fn folder(path: &Path, size: u64) -> Candidate {
        let p = path.to_string_lossy().into_owned();
        Candidate::new(Target::Folder { path: p.clone() }, "f".into(), p, size, String::new(), Action::Folder(path.to_path_buf()))
    }

    fn cache_owning(cache_type: CacheType, dir: &Path, size: u64) -> Candidate {
        let mut c = Candidate::new(
            Target::Cache { cache_type: cache_type.clone() },
            "c".into(),
            String::new(),
            size,
            String::new(),
            Action::Cache(cache_type),
        );
        c.owned_dirs = vec![dir.to_path_buf()];
        c
    }

    #[test]
    fn test_target_ids_are_stable() {
        assert_eq!(Target::Cache { cache_type: CacheType::DockerBuildCache }.id(), "cache:DockerBuildCache");
        let a = Target::Artifact { path: "wsl:/home/me/a/node_modules".into(), artifact_kind: "node_modules".into(), project: "/home/me/a".into() };
        assert_eq!(a.id(), "artifact:wsl:/home/me/a/node_modules");
        assert_eq!(Target::Folder { path: "/x".into() }.id(), "folder:/x");
        assert_eq!(Target::Custom { id: "k".into() }.id(), "custom:k");
    }

    #[test]
    fn test_cache_ids_resolve_back_to_types() {
        for cache_type in paths::supported_types() {
            let id = Target::Cache { cache_type: cache_type.clone() }.id();
            let parsed = CacheType::from_str(id.strip_prefix("cache:").unwrap());
            assert_eq!(parsed.as_ref(), Ok(cache_type), "{id}");
        }
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(Target::Cache { cache_type: CacheType::DockerVolumes }.risk(), Risk::Danger);
        assert_eq!(Target::Cache { cache_type: CacheType::RecycleBin }.risk(), Risk::Review);
        assert_eq!(Target::Cache { cache_type: CacheType::Npm }.risk(), Risk::Safe);
        let artifact = |k: &str| Target::Artifact { path: "p".into(), artifact_kind: k.into(), project: "q".into() };
        assert_eq!(artifact("node_modules").risk(), Risk::Safe);
        assert_eq!(artifact(".venv").risk(), Risk::Review);
        assert_eq!(artifact(artifacts::SWEEP_KIND).risk(), Risk::Review);
        assert_eq!(Target::Folder { path: "x".into() }.risk(), Risk::Review);
    }

    #[test]
    fn test_target_serialization_roundtrip() {
        let t = Target::Artifact { path: "wsl:/a".into(), artifact_kind: "target".into(), project: "/a".into() };
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"kind\":\"artifact\""), "{json}");
        assert_eq!(serde_json::from_str::<Target>(&json).unwrap(), t);
    }

    #[test]
    fn test_dedupe_roots_drops_nested() {
        let roots = vec![PathBuf::from("/a/b/c"), PathBuf::from("/a"), PathBuf::from("/z"), PathBuf::from("/a/b")];
        assert_eq!(dedupe_roots(roots), [PathBuf::from("/a"), PathBuf::from("/z")]);
    }

    #[test]
    fn test_finalize_marks_nested_and_dedupes() {
        let dir = tempfile::tempdir().unwrap();
        let npm = dir.path().join("npm-cache");
        let inner = npm.join("_cacache");
        let outside = dir.path().join("other");
        let list = finalize(vec![
            folder(&inner, 10),
            folder(&inner, 30),
            folder(&outside, 20),
            cache_owning(CacheType::Npm, &npm, 50),
        ]);
        let sizes: Vec<u64> = list.iter().map(|c| c.size_bytes).collect();
        assert_eq!(sizes, [50, 30, 20], "deduped (largest kept) and sorted");
        let nested = list.iter().find(|c| c.size_bytes == 30).unwrap();
        assert_eq!(nested.covered_by.as_deref(), Some("cache:Npm"));
        assert!(list.iter().find(|c| c.size_bytes == 20).unwrap().covered_by.is_none());
        assert!(list[0].covered_by.is_none(), "a cache never covers itself");
    }

    #[cfg(windows)]
    #[test]
    fn test_nesting_ignores_case_and_separators_on_windows() {
        assert!(path_within(Path::new(r"C:\Users\me\AppData/Local\uv\Cache\builds-v0"), Path::new(r"C:\Users\me\AppData\Local\uv\cache")));
        assert!(!path_within(Path::new(r"C:\Users\me\AppData\Local\uvx"), Path::new(r"C:\Users\me\AppData\Local\uv")));
        assert_eq!(normalize(Path::new(r"C:\a/b\c")), PathBuf::from(r"C:\a\b\c"));
    }

    #[test]
    fn test_selection_bytes_skips_covered_children() {
        let dir = tempfile::tempdir().unwrap();
        let npm = dir.path().join("npm");
        let list = finalize(vec![folder(&npm.join("x"), 30), cache_owning(CacheType::Npm, &npm, 50)]);
        let all: Vec<&Candidate> = list.iter().collect();
        assert_eq!(selection_bytes(&all), 50);
        let child_only: Vec<&Candidate> = list.iter().filter(|c| c.size_bytes == 30).collect();
        assert_eq!(selection_bytes(&child_only), 30);
    }

    #[tokio::test]
    async fn test_clean_selected_skips_child_of_selected_parent() {
        let dir = tempfile::tempdir().unwrap();
        let npm = dir.path().join("npm");
        let list = finalize(vec![folder(&npm.join("x"), 30), cache_owning(CacheType::Npm, &npm, 50)]);
        let child = list.iter().find(|c| c.size_bytes == 30).unwrap();
        let parent = list.iter().find(|c| c.size_bytes == 50).unwrap();
        // dry run so the real npm cache is not touched
        let outcomes = clean_selected(&[parent, child], true, |_, _| {}).await;
        assert_eq!(outcomes[1].message, "included in cache:Npm");
        assert_eq!(outcomes[1].freed_bytes, 0);
    }

    #[tokio::test]
    async fn test_clean_folder_removes_it_and_tolerates_missing() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("stale");
        std::fs::create_dir_all(target.join("deep")).unwrap();
        std::fs::write(target.join("deep/f.bin"), vec![0u8; 64]).unwrap();
        let c = folder(&target, 64);

        let dry = clean_selected(&[&c], true, |_, _| {}).await;
        assert!(target.exists() && dry[0].ok && dry[0].freed_bytes == 64);

        let real = clean_selected(&[&c], false, |_, _| {}).await;
        assert!(real[0].ok, "{:?}", real[0]);
        assert!(!target.exists());
        let again = clean_selected(&[&c], false, |_, _| {}).await;
        assert_eq!((again[0].ok, again[0].message.as_str()), (true, "already gone"));
    }

    #[test]
    fn test_remembered_folder_must_still_be_under_a_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("Caches");
        std::fs::create_dir_all(root.join("app")).unwrap();
        let roots = vec![root.clone()];
        assert!(is_suggestable_folder(&root.join("app"), &roots));
        assert!(!is_suggestable_folder(&root.join("missing"), &roots));
        assert!(!is_suggestable_folder(dir.path(), &roots), "parent of a root is not suggestable");
        assert!(!is_suggestable_folder(&root, &roots), "the root itself is not suggestable");
    }

    #[test]
    fn test_plan_for_ids() {
        let mut memory = Memory::default();
        let folder = Target::Folder { path: "/c/app".into() };
        memory.observe(&folder.id(), &folder, "app", "/c/app", 1, 1);

        let plan = plan_for_ids(&["cache:npm".into(), folder.id()], &opts(), &memory).unwrap();
        assert_eq!(plan.cache_types, [CacheType::Npm]);
        assert_eq!(plan.folders, FolderScan::Only(vec![PathBuf::from("/c/app")]));
        assert!(plan.local_roots.is_empty() && !plan.custom);

        let err = plan_for_ids(&["folder:/etc".into(), "cache:Nope".into()], &opts(), &memory).unwrap_err();
        assert_eq!(err, ["folder:/etc", "cache:Nope"], "unremembered paths are refused");
    }

    #[test]
    fn test_quick_plan_uses_only_recent_memory() {
        let mut memory = Memory::default();
        let npm = Target::Cache { cache_type: CacheType::Npm };
        let old = Target::Cache { cache_type: CacheType::Pip };
        memory.observe(&npm.id(), &npm, "npm", "", 1, 100 * 86_400);
        memory.observe(&old.id(), &old, "pip", "", 1, 0);
        let plan = quick_plan(&opts(), &memory, 100 * 86_400);
        assert_eq!(plan.cache_types, [CacheType::Npm]);
        assert_eq!(plan.folders, FolderScan::None);
    }
}
