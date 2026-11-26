//! `suggest`: scan → rank → tick checkboxes → clean → learn.
//!
//! Modes:
//! - interactive (terminal): checkbox list, pre-ticked from memory, then a confirmation;
//! - `--json`: print the suggestions and exit (what the Claude skill reads); nothing is deleted;
//! - `--select ID…`: clean exactly these ids (re-measured first) without prompting;
//! - `--yes`: clean what is pre-ticked without prompting.

use crate::view::{self, History, HOT};
use crate::{classic, memory_cmd};
use cache_cleaner::cache::advisor::{self, Candidate, Outcome, PlanOptions, Risk, Source, SourceState};
use cache_cleaner::cache::memory::{self, Advice, Decision, Memory, MIB};
use cache_cleaner::cache::CacheType;
use cache_cleaner::utils::format;
use clap::Args;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, MultiSelect};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::io::IsTerminal;
use std::process::ExitCode;
use std::time::{Duration, Instant};

#[derive(Args, Debug)]
pub struct SuggestArgs {
    /// Re-measure only the places remembered from earlier runs (fast)
    #[arg(long)]
    quick: bool,
    /// Print suggestions (or, with --select/--yes, results) as JSON; never prompts
    #[arg(long)]
    json: bool,
    /// Clean this suggestion id without prompting (repeatable). Ids come from `--json` output
    #[arg(long, value_name = "ID")]
    select: Vec<String>,
    /// Clean the pre-ticked suggestions without prompting
    #[arg(long, short = 'y', conflicts_with = "select")]
    yes: bool,
    /// Report what would be freed; delete nothing
    #[arg(long)]
    dry_run: bool,
    /// Hide suggestions smaller than this many MB
    #[arg(long, default_value_t = 100)]
    min_mb: u64,
    /// A project is idle (its build output suggested) when no file changed for this many days
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
    idle_days: u64,
    /// Also suggest files unread for N days inside active cargo targets (0 = off)
    #[arg(long, default_value_t = 0)]
    sweep_days: u64,
    /// Extra folder to search for build output (repeatable); `wsl:/path` searches inside WSL
    #[arg(long = "root", value_name = "PATH")]
    roots: Vec<String>,
    /// Do not search for build output of idle projects
    #[arg(long)]
    no_artifacts: bool,
    /// Leave WSL alone (no WSL caches, no build output inside WSL)
    #[arg(long)]
    no_wsl: bool,
}

#[derive(Serialize)]
struct Suggestion {
    #[serde(flatten)]
    candidate: Candidate,
    size: String,
    #[serde(flatten)]
    advice: Advice,
    history: Option<History>,
}

#[derive(Serialize)]
struct JsonReport<'a> {
    mode: &'a str,
    dry_run: bool,
    scan_seconds: f64,
    memory_path: String,
    total_bytes: u64,
    preselected_bytes: u64,
    suggestions: &'a [Suggestion],
    #[serde(skip_serializing_if = "Option::is_none")]
    results: Option<&'a [Outcome]>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    not_found: Vec<String>,
}

pub async fn run(args: SuggestArgs) -> ExitCode {
    let (path, mut memory) = match memory_cmd::open() {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    let now = memory::now();
    let interactive = !args.json && std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let opts = PlanOptions {
        idle_days: args.idle_days,
        sweep_days: args.sweep_days,
        min_bytes: args.min_mb * MIB,
        extra_roots: args.roots.clone(),
        artifacts: !args.no_artifacts,
        wsl: !args.no_wsl,
    };
    let select: BTreeSet<String> = args.select.iter().map(|id| canonical_id(id)).collect();

    // 1. Plan: explicit ids, remembered places only, or everything.
    let (plan, mode) = if !select.is_empty() {
        let ids: Vec<String> = select.iter().cloned().collect();
        match advisor::plan_for_ids(&ids, &opts, &memory) {
            Ok(plan) => (plan, "select"),
            Err(unknown) => {
                eprintln!(
                    "unknown suggestion id(s): {}\nrun `cache-cleaner-cli suggest --json` first so they are remembered",
                    unknown.join(", ")
                );
                return ExitCode::from(2);
            }
        }
    } else if args.quick && !memory.entries.is_empty() {
        (advisor::quick_plan(&opts, &memory, now), "quick")
    } else {
        if args.quick && !args.json {
            eprintln!("Nothing remembered yet: running a full scan this time.");
        }
        (advisor::full_plan(&opts, &memory), "full")
    };

    // 2. Scan.
    let started = Instant::now();
    let show_progress = !args.json && std::io::stderr().is_terminal();
    let candidates = scan_with_progress(&plan, show_progress).await;
    let scan_time = started.elapsed();

    // 3. Learn sizes; rank.
    memory.begin_run(now);
    for c in &candidates {
        if c.size_bytes >= opts.min_bytes || memory.entries.contains_key(&c.id) {
            memory.observe(&c.id, &c.target, &c.label, &c.location, c.size_bytes, now);
        }
    }
    let mut list: Vec<Suggestion> = candidates
        .into_iter()
        .filter(|c| if select.is_empty() { c.size_bytes >= opts.min_bytes } else { select.contains(&c.id) })
        .map(|c| suggestion(c, &memory))
        .collect();
    list.sort_by(|a, b| {
        (b.advice.preselect, b.advice.hot, b.candidate.size_bytes)
            .cmp(&(a.advice.preselect, a.advice.hot, a.candidate.size_bytes))
    });
    let report = |list: &[Suggestion], results: Option<&[Outcome]>, not_found: Vec<String>| {
        print_json(&JsonReport {
            mode,
            dry_run: args.dry_run,
            scan_seconds: scan_time.as_secs_f64(),
            memory_path: path.display().to_string(),
            total_bytes: unique_bytes(list, |_| true),
            preselected_bytes: unique_bytes(list, |s| s.advice.preselect),
            suggestions: list,
            results,
            not_found,
        })
    };

    // 4. Decide what to clean, and what counts as "offered" for learning.
    let mut not_found = Vec::new();
    let (picked, offered): (BTreeSet<String>, BTreeSet<String>) = if !select.is_empty() {
        not_found = select.iter().filter(|id| !list.iter().any(|s| &s.candidate.id == *id)).cloned().collect();
        if !not_found.is_empty() && !args.json {
            eprintln!(
                "no longer suggested (already gone, project active again, or not supported): {}",
                not_found.join(", ")
            );
        }
        let mut offered: BTreeSet<String> = memory.take_pending_offer(now).into_iter().collect();
        offered.extend(list.iter().map(|s| s.candidate.id.clone()));
        (ids_of(&list), offered)
    } else if args.yes {
        let picked = list.iter().filter(|s| s.advice.preselect).map(|s| s.candidate.id.clone()).collect();
        (picked, ids_of(&list))
    } else if interactive {
        if list.is_empty() {
            println!("Nothing over {} worth cleaning. Disk looks tidy.", format::size(opts.min_bytes));
            return finish(&path, &mut memory, now, ExitCode::SUCCESS);
        }
        print_header(&list, scan_time, &memory);
        match pick(&list) {
            Some(picked) => (picked, ids_of(&list)),
            None => {
                println!("Cancelled; nothing deleted.");
                return finish(&path, &mut memory, now, ExitCode::SUCCESS);
            }
        }
    } else {
        // Listing only (JSON or piped output): remember what was shown for a later --select.
        memory.offer(list.iter().map(|s| s.candidate.id.clone()).collect(), now);
        if args.json {
            report(&list, None, Vec::new());
        } else {
            print_plain(&list);
        }
        return finish(&path, &mut memory, now, ExitCode::SUCCESS);
    };

    // 5. Confirm and clean.
    let selected: Vec<&Candidate> =
        list.iter().map(|s| &s.candidate).filter(|c| picked.contains(&c.id)).collect();
    if selected.is_empty() {
        if !args.json {
            println!("Nothing selected; nothing deleted.");
        }
    } else if interactive && select.is_empty() && !args.yes && !args.dry_run && !confirm(&selected) {
        println!("Nothing deleted.");
        return finish(&path, &mut memory, now, ExitCode::SUCCESS);
    }
    let outcomes = clean_with_progress(&selected, args.dry_run, !args.json).await;

    // 6. Learn the decision (not from dry runs: nothing was really chosen).
    if !args.dry_run {
        let freed: HashMap<&str, u64> = outcomes.iter().map(|o| (o.id.as_str(), o.freed_bytes)).collect();
        for id in &offered {
            let decision = match freed.get(id.as_str()) {
                Some(&freed) => Decision::Cleaned { freed },
                None => Decision::Kept,
            };
            memory.record(id, decision, now);
        }
    }

    let failed = outcomes.iter().filter(|o| !o.ok).count();
    if args.json {
        report(&list, Some(&outcomes), not_found);
    } else if !selected.is_empty() {
        print_summary(&outcomes, failed, args.dry_run, &memory);
    }
    let code = if failed == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    finish(&path, &mut memory, now, code)
}

fn finish(path: &std::path::Path, memory: &mut Memory, now: u64, code: ExitCode) -> ExitCode {
    memory.prune(now);
    if let Err(err) = memory.save(path) {
        eprintln!("warning: could not save memory to {}: {err}", path.display());
    }
    code
}

/// `cache:npm` → `cache:Npm`, so ids typed by hand match the canonical ones.
fn canonical_id(id: &str) -> String {
    match id.strip_prefix("cache:").map(CacheType::from_str) {
        Some(Ok(cache_type)) => format!("cache:{cache_type:?}"),
        _ => id.to_string(),
    }
}

/// Bytes of the matching suggestions, nested ones counted once (via their parent).
fn unique_bytes(list: &[Suggestion], keep: impl Fn(&Suggestion) -> bool) -> u64 {
    let chosen: Vec<&Candidate> = list.iter().filter(|s| keep(s)).map(|s| &s.candidate).collect();
    advisor::selection_bytes(&chosen)
}

fn ids_of(list: &[Suggestion]) -> BTreeSet<String> {
    list.iter().map(|s| s.candidate.id.clone()).collect()
}

fn suggestion(candidate: Candidate, memory: &Memory) -> Suggestion {
    let mut advice = memory.advise(&candidate.id, candidate.risk, candidate.size_bytes);
    if candidate.covered_by.is_some() {
        // Its parent cache is the better pick; never tick both.
        advice.preselect = false;
    }
    Suggestion {
        size: format::size(candidate.size_bytes),
        history: memory.entries.get(&candidate.id).map(History::from),
        advice,
        candidate,
    }
}

async fn scan_with_progress(plan: &advisor::ScanPlan, visible: bool) -> Vec<Candidate> {
    let multi = MultiProgress::with_draw_target(if visible {
        ProgressDrawTarget::stderr()
    } else {
        ProgressDrawTarget::hidden()
    });
    let spinner = ProgressStyle::with_template("{spinner:.cyan} {prefix:<22} {msg}")
        .expect("valid template")
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]);
    let bars: HashMap<Source, ProgressBar> = Source::ALL
        .into_iter()
        .filter(|s| s.enabled(plan))
        .map(|source| {
            let bar = multi.add(ProgressBar::new_spinner().with_style(spinner.clone()));
            bar.set_prefix(source.name());
            bar.set_message(style("waiting").dim().to_string());
            (source, bar)
        })
        .collect();

    let on_progress = |source: Source, state: SourceState| {
        let Some(bar) = bars.get(&source) else { return };
        match state {
            SourceState::Started => {
                bar.set_message("scanning…");
                bar.enable_steady_tick(Duration::from_millis(100));
            }
            SourceState::Done { found, bytes, elapsed } => bar.finish_with_message(format!(
                "{found} found · {} {}",
                format::size(bytes),
                style(format!("({:.1}s)", elapsed.as_secs_f64())).dim()
            )),
            SourceState::Failed(err) => {
                bar.finish_with_message(style(format!("skipped: {err}")).dim().to_string())
            }
        }
    };
    let candidates = advisor::collect(plan, &on_progress).await;
    let _ = multi.clear();
    candidates
}

fn print_header(list: &[Suggestion], scan_time: Duration, memory: &Memory) {
    let total = unique_bytes(list, |_| true);
    let ticked: Vec<&Candidate> = list.iter().filter(|s| s.advice.preselect).map(|s| &s.candidate).collect();
    let learned = if memory.runs > 1 {
        format!("memory: {} runs, {} places", memory.runs, memory.entries.len())
    } else {
        "first run: learning starts now".to_string()
    };
    println!(
        "\n{} {} suggestions · {} {}",
        style("Found").bold(),
        list.len(),
        style(format::size(total)).bold(),
        style(format!("in {:.1}s · {learned}", scan_time.as_secs_f64())).dim()
    );
    println!(
        "{}",
        style(format!(
            "{HOT} = hot spot (often large / fills up fast) · pre-ticked {} ({}) = learned from your choices or large & safe",
            ticked.len(),
            format::size(advisor::selection_bytes(&ticked))
        ))
        .dim()
    );
    println!(
        "{} {} {}\n",
        style("risk:").dim(),
        view::risk_badge(Risk::Safe),
        style(format!(
            "rebuilt on demand · {} review first · {} may lose data",
            view::risk_badge(Risk::Review),
            view::risk_badge(Risk::Danger)
        ))
        .dim()
    );
}

/// Checkbox list. `None` when the user cancels (Esc / q).
fn pick(list: &[Suggestion]) -> Option<BTreeSet<String>> {
    let width = view::terminal_width().saturating_sub(4);
    let items: Vec<String> = list.iter().map(|s| view::row(&s.candidate, &s.advice, width)).collect();
    let defaults: Vec<bool> = list.iter().map(|s| s.advice.preselect).collect();
    let page = (console::Term::stdout().size().0 as usize).saturating_sub(12).clamp(5, 25);
    let picked = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("↑↓ move · space tick · a all · enter delete ticked · esc cancel")
        .items(&items)
        .defaults(&defaults)
        .max_length(page)
        .report(false)
        .interact_opt();
    match picked {
        Ok(Some(indexes)) => Some(indexes.into_iter().map(|i| list[i].candidate.id.clone()).collect()),
        Ok(None) => None,
        Err(err) => {
            eprintln!("cannot show the checkbox list: {err}");
            None
        }
    }
}

fn confirm(selected: &[&Candidate]) -> bool {
    let danger: Vec<&&Candidate> = selected.iter().filter(|c| c.risk == Risk::Danger).collect();
    for c in &danger {
        println!("{} {} — {}", style("DATA").red().bold(), c.label, c.note);
    }
    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Delete {} item(s), freeing about {}?",
            selected.len(),
            format::size(advisor::selection_bytes(selected))
        ))
        .default(danger.is_empty())
        .interact_opt()
        .ok()
        .flatten()
        .unwrap_or(false)
}

async fn clean_with_progress(selected: &[&Candidate], dry_run: bool, visible: bool) -> Vec<Outcome> {
    if selected.is_empty() {
        return Vec::new();
    }
    let bar = ProgressBar::with_draw_target(
        Some(selected.len() as u64),
        if visible { ProgressDrawTarget::stderr() } else { ProgressDrawTarget::hidden() },
    );
    bar.set_style(ProgressStyle::with_template("{bar:30.cyan/blue} {pos}/{len} {msg}").expect("valid template"));
    bar.set_message(if dry_run { "measuring…" } else { "cleaning…" });
    let outcomes = advisor::clean_selected(selected, dry_run, |candidate, outcome| {
        if visible {
            let mark = if outcome.ok { style("✓").green() } else { style("✗").red() };
            bar.println(format!(
                "{mark} {:>9}  {}  {}",
                format::size(outcome.freed_bytes),
                candidate.label,
                style(&outcome.message).dim()
            ));
        }
        bar.inc(1);
    })
    .await;
    bar.finish_and_clear();
    outcomes
}

fn print_summary(outcomes: &[Outcome], failed: usize, dry_run: bool, memory: &Memory) {
    let freed: u64 = outcomes.iter().map(|o| o.freed_bytes).sum();
    let verb = if dry_run { "Would free" } else { "Freed" };
    let failures = if failed > 0 { format!(", {failed} failed") } else { String::new() };
    println!(
        "\n{} {} ({} item(s){failures})",
        style(verb).bold(),
        style(format::size(freed)).green().bold(),
        outcomes.len()
    );
    if dry_run {
        return;
    }
    let next: usize = memory
        .entries
        .iter()
        .filter(|(id, e)| memory.advise(id, e.target.risk(), e.last_bytes()).preselect)
        .count();
    println!(
        "{}",
        style(format!(
            "Learned: {next} place(s) will be pre-ticked next time · {} hot spot(s). \
             See `cache-cleaner-cli memory`; `suggest --quick` re-checks only them.",
            memory.hot_spots().len()
        ))
        .dim()
    );
    classic::warn_if_unprivileged();
}

/// Piped output without --json: readable list plus ids for `--select`.
fn print_plain(list: &[Suggestion]) {
    for s in list {
        let tick = if s.advice.preselect { "[x]" } else { "[ ]" };
        let hot = if s.advice.hot { " hot" } else { "" };
        println!(
            "{tick} {:>9}  {:?}{hot}  {}  {}",
            s.size, s.candidate.risk, s.candidate.label, s.advice.reason
        );
        println!("      id: {}", s.candidate.id);
    }
    let total = unique_bytes(list, |_| true);
    println!(
        "{} suggestion(s), {} total. Clean with: cache-cleaner-cli suggest --select <id> ...",
        list.len(),
        format::size(total)
    );
}

fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("cannot encode JSON: {err}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_id() {
        assert_eq!(canonical_id("cache:npm"), "cache:Npm");
        assert_eq!(canonical_id("cache:docker_build_cache"), "cache:DockerBuildCache");
        assert_eq!(canonical_id("cache:bogus"), "cache:bogus");
        assert_eq!(canonical_id("folder:/x"), "folder:/x");
    }
}
