//! `memory`: what the cleaner learned (hot spots, habits), and ways to correct it.

use crate::view::{self, History};
use cache_cleaner::cache::advisor::Risk;
use cache_cleaner::cache::memory::{self, Entry, Memory};
use cache_cleaner::utils::format;
use clap::{Args, Subcommand};
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirm;
use serde::Serialize;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Args, Debug)]
pub struct MemoryArgs {
    #[command(subcommand)]
    action: Option<MemoryAction>,
    /// Machine-readable output
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand, Debug)]
enum MemoryAction {
    /// Hot spots, what will be pre-ticked next time, what you usually keep (default)
    Show,
    /// Forget one place (its id from `suggest --json` or `memory --json`)
    Forget { id: String },
    /// Forget everything
    Reset {
        /// Do not ask for confirmation
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Print where the memory file lives
    Path,
}

/// Loads the memory file. A corrupt file is moved aside (never silently overwritten).
pub fn open() -> Result<(PathBuf, Memory), ExitCode> {
    let path = memory::default_path().map_err(|err| {
        eprintln!("{err}");
        ExitCode::FAILURE
    })?;
    match Memory::load(&path) {
        Ok(memory) => Ok((path, memory)),
        Err(err) => {
            let aside = path.with_extension(format!("corrupt-{}.json", memory::now()));
            eprintln!("warning: {err:#}; moved to {} and starting fresh", aside.display());
            if std::fs::rename(&path, &aside).is_err() {
                eprintln!("cannot move the corrupt file aside; fix or delete {}", path.display());
                return Err(ExitCode::FAILURE);
            }
            Ok((path, Memory::default()))
        }
    }
}

pub fn run(args: MemoryArgs) -> ExitCode {
    let (path, mut memory) = match open() {
        Ok(opened) => opened,
        Err(code) => return code,
    };
    match args.action.unwrap_or(MemoryAction::Show) {
        MemoryAction::Show if args.json => {
            print_json(&path, &memory);
            ExitCode::SUCCESS
        }
        MemoryAction::Show => {
            show(&path, &memory);
            ExitCode::SUCCESS
        }
        MemoryAction::Path => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        MemoryAction::Forget { id } => {
            if !memory.forget(&id) {
                eprintln!("{id} is not remembered (see `cache-cleaner-cli memory --json`)");
                return ExitCode::from(2);
            }
            save(&path, &memory, &format!("Forgot {id}."))
        }
        MemoryAction::Reset { yes } => {
            let confirmed = yes
                || (std::io::stdin().is_terminal()
                    && Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Forget all {} remembered places?", memory.entries.len()))
                        .default(false)
                        .interact()
                        .unwrap_or(false));
            if !confirmed {
                eprintln!("Not reset (pass --yes to skip the question).");
                return ExitCode::from(2);
            }
            match std::fs::remove_file(&path) {
                Ok(()) => println!("Memory reset."),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => println!("Memory was already empty."),
                Err(e) => {
                    eprintln!("cannot delete {}: {e}", path.display());
                    return ExitCode::FAILURE;
                }
            }
            ExitCode::SUCCESS
        }
    }
}

fn save(path: &Path, memory: &Memory, message: &str) -> ExitCode {
    match memory.save(path) {
        Ok(()) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("cannot save {}: {err}", path.display());
            ExitCode::FAILURE
        }
    }
}

/// One remembered place with what memory concludes about it.
#[derive(Serialize)]
struct Place<'a> {
    id: &'a str,
    label: &'a str,
    location: &'a str,
    risk: Risk,
    hot: bool,
    preselect_next: bool,
    reason: String,
    last_bytes: u64,
    last_seen_ago: String,
    #[serde(flatten)]
    history: History,
}

fn places(memory: &Memory) -> Vec<Place<'_>> {
    let now = memory::now();
    let mut list: Vec<Place> = memory
        .entries
        .iter()
        .map(|(id, e)| {
            let advice = memory.advise(id, e.target.risk(), e.last_bytes());
            Place {
                id,
                label: &e.label,
                location: &e.location,
                risk: e.target.risk(),
                hot: advice.hot,
                preselect_next: advice.preselect,
                reason: advice.reason,
                last_bytes: e.last_bytes(),
                last_seen_ago: format::ago(now.saturating_sub(e.last_seen)),
                history: History::from(e),
            }
        })
        .collect();
    list.sort_by_key(|p| std::cmp::Reverse(p.history.avg_bytes));
    list
}

fn print_json(path: &Path, memory: &Memory) {
    #[derive(Serialize)]
    struct Report<'a> {
        path: String,
        runs: u32,
        last_run_ago: Option<String>,
        places: Vec<Place<'a>>,
    }
    let now = memory::now();
    let report = Report {
        path: path.display().to_string(),
        runs: memory.runs,
        last_run_ago: memory.last_run.map(|t| format::ago(now.saturating_sub(t))),
        places: places(memory),
    };
    match serde_json::to_string_pretty(&report) {
        Ok(json) => println!("{json}"),
        Err(err) => eprintln!("cannot encode JSON: {err}"),
    }
}

fn show(path: &Path, memory: &Memory) {
    let now = memory::now();
    let last = memory.last_run.map(|t| format::ago(now.saturating_sub(t))).unwrap_or_else(|| "never".into());
    println!(
        "{} {}  {}",
        style("Memory").bold(),
        path.display(),
        style(format!("· {} run(s) · last {last} · {} place(s)", memory.runs, memory.entries.len())).dim()
    );
    if memory.entries.is_empty() {
        println!("Nothing learned yet. Run `cache-cleaner-cli suggest`.");
        return;
    }
    let list = places(memory);
    let width = view::terminal_width().max(60);
    section("Hot spots (often large or filling up fast)", list.iter().filter(|p| p.hot), width);
    section("Pre-ticked next time", list.iter().filter(|p| p.preselect_next), width);
    section(
        "You usually keep",
        list.iter().filter(|p| !p.preselect_next && p.history.times_kept > p.history.times_cleaned),
        width,
    );
    let freed: u64 = memory.entries.values().map(|e: &Entry| e.total_freed).sum();
    println!("\n{} {}", style("Freed through suggest so far:").dim(), style(format::size(freed)).green());
}

fn section<'a>(title: &str, places: impl Iterator<Item = &'a Place<'a>>, width: usize) {
    let places: Vec<&Place> = places.collect();
    if places.is_empty() {
        return;
    }
    println!("\n{}", style(title).bold().underlined());
    for p in places {
        let mut facts = vec![format!("seen {}×", p.history.times_seen)];
        if p.history.times_cleaned + p.history.times_kept > 0 {
            facts.push(format!("cleaned {}/{}", p.history.times_cleaned, p.history.times_cleaned + p.history.times_kept));
        }
        if let Some(rate) = p.history.regrow_bytes_per_day {
            facts.push(format!("regrows ~{}/day", format::size(rate)));
        }
        facts.push(p.last_seen_ago.clone());
        let line = format!(
            "  {:>9} avg  {}  {}  {}",
            format::size(p.history.avg_bytes),
            view::risk_badge(p.risk),
            console::pad_str(p.label, 34, console::Alignment::Left, Some("…")),
            style(facts.join(" · ")).dim()
        );
        println!("{}", console::truncate_str(&line, width, "…"));
    }
}
