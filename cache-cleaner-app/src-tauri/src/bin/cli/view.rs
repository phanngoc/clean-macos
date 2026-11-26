//! Terminal rendering shared by `suggest` and `memory`.

use cache_cleaner::cache::advisor::{Candidate, Risk};
use cache_cleaner::cache::memory::{Advice, Entry};
use cache_cleaner::utils::format;
use console::{style, Emoji, StyledObject};
use serde::Serialize;

pub static HOT: Emoji<'_, '_> = Emoji("🔥", "*");
const LABEL_WIDTH: usize = 34;

pub fn risk_badge(risk: Risk) -> StyledObject<&'static str> {
    match risk {
        Risk::Safe => style("safe ").green(),
        Risk::Review => style("check").yellow(),
        Risk::Danger => style("DATA ").red().bold(),
    }
}

/// One checkbox line: `🔥   12.3 GB  safe   Docker build cache   · you cleaned this 3 of 3 times`.
pub fn row(candidate: &Candidate, advice: &Advice, width: usize) -> String {
    let hot = if advice.hot { HOT.to_string() } else { "  ".to_string() };
    let label = console::pad_str(&candidate.label, LABEL_WIDTH, console::Alignment::Left, Some("…"));
    let detail = match &candidate.covered_by {
        Some(parent) => format!(" · inside {parent}"),
        None if !advice.reason.is_empty() => format!(" · {}", advice.reason),
        None => format!(" · {}", candidate.location),
    };
    let line = format!(
        "{hot} {:>9}  {}  {label}{}",
        format::size(candidate.size_bytes),
        risk_badge(candidate.risk),
        style(detail).dim()
    );
    console::truncate_str(&line, width.max(40), "…").into_owned()
}

/// What memory knows about a suggestion, as reported in JSON.
#[derive(Debug, Serialize)]
pub struct History {
    pub times_seen: u32,
    pub times_cleaned: u32,
    pub times_kept: u32,
    pub avg_bytes: u64,
    pub total_freed: u64,
    pub regrow_bytes_per_day: Option<u64>,
}

impl From<&Entry> for History {
    fn from(e: &Entry) -> Self {
        Self {
            times_seen: e.times_seen,
            times_cleaned: e.times_cleaned,
            times_kept: e.times_kept,
            avg_bytes: e.avg_bytes(),
            total_freed: e.total_freed,
            regrow_bytes_per_day: e.regrow_per_day.map(|r| r as u64),
        }
    }
}

pub fn terminal_width() -> usize {
    console::Term::stdout().size().1 as usize
}
