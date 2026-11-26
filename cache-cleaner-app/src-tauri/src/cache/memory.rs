//! Self-learning memory behind `suggest`, persisted at `~/.cache-cleaner/memory.json`.
//!
//! Every run records how big each place was (size samples) and what the user decided for it
//! (cleaned or kept). From that history the advisor learns:
//!
//! - **hot spots**: places that are large scan after scan, or fill up again fast after a clean;
//! - **habits**: what the user keeps cleaning (pre-selected next time) and what they keep
//!   (no longer pre-selected).
//!
//! `suggest --quick` re-measures only the places remembered here instead of scanning everything.

use super::advisor::{Risk, Target};
use crate::utils::format;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MIB: u64 = 1024 * 1024;
pub const GIB: u64 = 1024 * MIB;
const HOUR: u64 = 3600;
const DAY: u64 = 24 * HOUR;

const MAX_SAMPLES: usize = 30;
const MAX_ENTRIES: usize = 500;
/// Places never cleaned and not seen for this long are forgotten.
const FORGET_AFTER: u64 = 120 * DAY;
/// `--quick` re-measures places seen within this window.
pub const QUICK_WINDOW: u64 = 60 * DAY;
/// A list printed by `suggest --json` counts as "offered" to a later `--select` for this long.
const OFFER_TTL: u64 = DAY;

/// Scans closer together than this belong to the same session.
const SESSION_GAP: u64 = 6 * HOUR;
/// Hot spot: at least this large on average over two or more sessions…
const HOT_AVG_BYTES: u64 = GIB;
/// …or filling up again at least this fast after being cleaned.
const HOT_REGROW_PER_DAY: f64 = 200.0 * MIB as f64;
/// Safe places this large are pre-selected even without history.
const AUTO_SELECT_BYTES: u64 = GIB;

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// `~/.cache-cleaner/memory.json`, next to the app's `config.json`.
pub fn default_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("cannot find home directory")?;
    Ok(home.join(".cache-cleaner").join("memory.json"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sample {
    pub at: u64,
    pub bytes: u64,
}

/// Everything learned about one place (keyed by suggestion id).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub target: Target,
    pub label: String,
    pub location: String,
    pub first_seen: u64,
    pub last_seen: u64,
    #[serde(default)]
    pub times_seen: u32,
    /// Size at each scan, oldest first (bounded).
    #[serde(default)]
    pub samples: Vec<Sample>,
    #[serde(default)]
    pub times_offered: u32,
    #[serde(default)]
    pub times_cleaned: u32,
    #[serde(default)]
    pub times_kept: u32,
    #[serde(default)]
    pub total_freed: u64,
    #[serde(default)]
    pub last_cleaned: Option<u64>,
    /// Smoothed bytes per day the place fills up again after a clean.
    #[serde(default)]
    pub regrow_per_day: Option<f64>,
}

impl Entry {
    fn new(target: Target, label: &str, location: &str, now: u64) -> Self {
        Self {
            target,
            label: label.to_string(),
            location: location.to_string(),
            first_seen: now,
            last_seen: now,
            times_seen: 0,
            samples: Vec::new(),
            times_offered: 0,
            times_cleaned: 0,
            times_kept: 0,
            total_freed: 0,
            last_cleaned: None,
            regrow_per_day: None,
        }
    }

    pub fn avg_bytes(&self) -> u64 {
        if self.samples.is_empty() {
            return 0;
        }
        self.samples.iter().map(|s| s.bytes).sum::<u64>() / self.samples.len() as u64
    }

    pub fn last_bytes(&self) -> u64 {
        self.samples.last().map_or(0, |s| s.bytes)
    }

    /// Share of offers the user accepted, Laplace-smoothed (0.5 without history).
    pub fn acceptance(&self) -> f64 {
        (self.times_cleaned as f64 + 1.0) / (self.times_offered as f64 + 2.0)
    }

    /// Scans at least [`SESSION_GAP`] apart: re-running `suggest` a few times in a row is one
    /// observation, not a pattern.
    pub fn sessions(&self) -> u32 {
        let mut count = 0;
        let mut last: Option<u64> = None;
        for sample in &self.samples {
            if last.is_none_or(|t| sample.at.saturating_sub(t) >= SESSION_GAP) {
                count += 1;
                last = Some(sample.at);
            }
        }
        count
    }

    /// Large across separate sessions, or quick to fill up again.
    pub fn is_hot(&self) -> bool {
        (self.sessions() >= 2 && self.avg_bytes() >= HOT_AVG_BYTES)
            || self.regrow_per_day.is_some_and(|r| r >= HOT_REGROW_PER_DAY)
    }

    fn hot_reason(&self) -> String {
        match self.regrow_per_day {
            Some(rate) if rate >= HOT_REGROW_PER_DAY => {
                format!("fills up again ~{}/day", format::size(rate as u64))
            }
            _ => format!("often large: avg {} over {} sessions", format::size(self.avg_bytes()), self.sessions()),
        }
    }
}

/// Suggestions printed by `suggest --json`, so a later `--select` knows what the user saw.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingOffer {
    pub at: u64,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Decision {
    Cleaned { freed: u64 },
    Kept,
}

/// Whether to pre-select a suggestion, and why.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Advice {
    pub preselect: bool,
    pub hot: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Memory {
    #[serde(default)]
    pub runs: u32,
    #[serde(default)]
    pub last_run: Option<u64>,
    #[serde(default)]
    pub entries: BTreeMap<String, Entry>,
    #[serde(default)]
    pub pending_offer: Option<PendingOffer>,
}

impl Memory {
    /// Empty memory when the file does not exist yet.
    pub fn load(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).with_context(|| format!("corrupt memory file {}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e).with_context(|| format!("cannot read {}", path.display())),
        }
    }

    /// Writes atomically (temp file + rename) so an interrupted run never leaves half a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn begin_run(&mut self, now: u64) {
        self.runs += 1;
        self.last_run = Some(now);
    }

    /// Records the current size of a place.
    pub fn observe(&mut self, id: &str, target: &Target, label: &str, location: &str, bytes: u64, now: u64) {
        let entry = self
            .entries
            .entry(id.to_string())
            .or_insert_with(|| Entry::new(target.clone(), label, location, now));
        entry.target = target.clone();
        entry.label = label.to_string();
        entry.location = location.to_string();

        // First measurement since the last clean tells how fast the place fills up again.
        if let Some(cleaned_at) = entry.last_cleaned {
            let first_since_clean = entry.samples.last().is_none_or(|s| s.at <= cleaned_at);
            let elapsed = now.saturating_sub(cleaned_at);
            if first_since_clean && elapsed >= HOUR {
                let rate = bytes as f64 * DAY as f64 / elapsed as f64;
                entry.regrow_per_day = Some(match entry.regrow_per_day {
                    Some(previous) => 0.5 * previous + 0.5 * rate,
                    None => rate,
                });
            }
        }

        entry.times_seen += 1;
        entry.last_seen = now;
        entry.samples.push(Sample { at: now, bytes });
        if entry.samples.len() > MAX_SAMPLES {
            entry.samples.remove(0);
        }
    }

    /// Records what the user did with an offered suggestion. Unknown ids are ignored.
    pub fn record(&mut self, id: &str, decision: Decision, now: u64) {
        let Some(entry) = self.entries.get_mut(id) else { return };
        entry.times_offered += 1;
        match decision {
            Decision::Cleaned { freed } => {
                entry.times_cleaned += 1;
                entry.total_freed += freed;
                entry.last_cleaned = Some(now);
            }
            Decision::Kept => entry.times_kept += 1,
        }
    }

    /// Pre-selection rule, from strongest to weakest signal:
    /// 1. `Danger` (user data) is never pre-selected;
    /// 2. habits: cleaned it at least twice (≥60% of offers), or cleaned it the only time a safe
    ///    place was offered → pre-select; kept it most of the time → don't;
    /// 3. safe and ≥ 1 GB → pre-select;
    /// 4. safe hot spot → pre-select.
    pub fn advise(&self, id: &str, risk: Risk, bytes: u64) -> Advice {
        let entry = self.entries.get(id);
        let hot = entry.is_some_and(Entry::is_hot);
        let hot_reason = || entry.map(Entry::hot_reason).unwrap_or_default();
        let advice = |preselect, reason: String| Advice { preselect, hot, reason };

        if risk == Risk::Danger {
            return advice(false, "may hold data you cannot get back; review first".to_string());
        }
        if let Some(e) = entry {
            let (cleaned, offered) = (e.times_cleaned, e.times_offered);
            if cleaned >= 2 && e.acceptance() >= 0.6 {
                return advice(true, format!("you cleaned this {cleaned} of {offered} times"));
            }
            if cleaned == 1 && e.times_kept == 0 && risk == Risk::Safe {
                return advice(true, "you cleaned this last time".to_string());
            }
            if offered >= 2 && e.acceptance() < 0.4 {
                return advice(false, format!("you kept this {} of {offered} times", e.times_kept));
            }
        }
        if risk == Risk::Safe && bytes >= AUTO_SELECT_BYTES {
            let reason = if hot { hot_reason() } else { "large and rebuilt on demand".to_string() };
            return advice(true, reason);
        }
        if hot {
            return advice(risk == Risk::Safe, hot_reason());
        }
        advice(false, String::new())
    }

    /// Remembers what `suggest --json` showed, for a later `--select`.
    pub fn offer(&mut self, ids: Vec<String>, now: u64) {
        self.pending_offer = Some(PendingOffer { at: now, ids });
    }

    /// Ids of the last `--json` listing if it is recent; clears it.
    pub fn take_pending_offer(&mut self, now: u64) -> Vec<String> {
        match self.pending_offer.take() {
            Some(offer) if now.saturating_sub(offer.at) <= OFFER_TTL => offer.ids,
            _ => Vec::new(),
        }
    }

    pub fn forget(&mut self, id: &str) -> bool {
        self.entries.remove(id).is_some()
    }

    /// Drops stale never-cleaned places and caps the number of entries.
    pub fn prune(&mut self, now: u64) {
        self.entries
            .retain(|_, e| e.times_cleaned > 0 || now.saturating_sub(e.last_seen) < FORGET_AFTER);
        if self.entries.len() > MAX_ENTRIES {
            let mut by_age: Vec<(u64, String)> =
                self.entries.iter().map(|(id, e)| (e.last_seen, id.clone())).collect();
            by_age.sort();
            for (_, id) in by_age.into_iter().take(self.entries.len() - MAX_ENTRIES) {
                self.entries.remove(&id);
            }
        }
    }

    /// Hot spots, largest average first.
    pub fn hot_spots(&self) -> Vec<(&String, &Entry)> {
        let mut hot: Vec<_> = self.entries.iter().filter(|(_, e)| e.is_hot()).collect();
        hot.sort_by_key(|(_, e)| std::cmp::Reverse(e.avg_bytes()));
        hot
    }

    /// Places seen within `window` seconds: what `--quick` re-measures.
    pub fn recent(&self, now: u64, window: u64) -> impl Iterator<Item = (&String, &Entry)> {
        self.entries.iter().filter(move |(_, e)| now.saturating_sub(e.last_seen) <= window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::CacheType;

    const T0: u64 = 1_700_000_000;

    fn npm() -> Target {
        Target::Cache { cache_type: CacheType::Npm }
    }

    fn seen(memory: &mut Memory, id: &str, bytes: u64, at: u64) {
        memory.observe(id, &npm(), "npm cache", "~/.npm", bytes, at);
    }

    #[test]
    fn test_first_run_preselects_only_large_safe_items() {
        let m = Memory::default();
        assert!(m.advise("a", Risk::Safe, 2 * GIB).preselect);
        assert!(!m.advise("a", Risk::Safe, 300 * MIB).preselect);
        assert!(!m.advise("a", Risk::Review, 5 * GIB).preselect);
        assert!(!m.advise("a", Risk::Danger, 50 * GIB).preselect);
    }

    #[test]
    fn test_learns_habit_of_cleaning() {
        let mut m = Memory::default();
        for run in 0..2 {
            seen(&mut m, "cache:Npm", 200 * MIB, T0 + run * DAY);
            m.record("cache:Npm", Decision::Cleaned { freed: 200 * MIB }, T0 + run * DAY);
        }
        let advice = m.advise("cache:Npm", Risk::Review, 200 * MIB);
        assert!(advice.preselect, "{advice:?}");
        assert_eq!(advice.reason, "you cleaned this 2 of 2 times");
    }

    #[test]
    fn test_single_clean_of_safe_item_is_remembered() {
        let mut m = Memory::default();
        seen(&mut m, "x", 200 * MIB, T0);
        m.record("x", Decision::Cleaned { freed: 1 }, T0);
        assert!(m.advise("x", Risk::Safe, 200 * MIB).preselect);
        assert!(!m.advise("x", Risk::Review, 200 * MIB).preselect);
    }

    #[test]
    fn test_learns_habit_of_keeping_even_large_items() {
        let mut m = Memory::default();
        for run in 0..3 {
            seen(&mut m, "big", 5 * GIB, T0 + run * DAY);
            m.record("big", Decision::Kept, T0 + run * DAY);
        }
        let advice = m.advise("big", Risk::Safe, 5 * GIB);
        assert!(!advice.preselect);
        assert_eq!(advice.reason, "you kept this 3 of 3 times");
    }

    #[test]
    fn test_danger_is_never_preselected_even_when_always_cleaned() {
        let mut m = Memory::default();
        for run in 0..5 {
            seen(&mut m, "vol", GIB, T0 + run * DAY);
            m.record("vol", Decision::Cleaned { freed: GIB }, T0 + run * DAY);
        }
        assert!(!m.advise("vol", Risk::Danger, GIB).preselect);
    }

    #[test]
    fn test_hot_spot_from_repeated_large_scans() {
        let mut m = Memory::default();
        seen(&mut m, "d", 3 * GIB, T0);
        assert!(!m.entries["d"].is_hot(), "one scan is not a pattern");
        seen(&mut m, "d", 3 * GIB, T0 + 600);
        assert!(!m.entries["d"].is_hot(), "re-running minutes later is the same session");
        seen(&mut m, "d", GIB / 2, T0 + DAY);
        assert_eq!(m.entries["d"].sessions(), 2);
        assert!(m.entries["d"].is_hot()); // avg 2.2 GB over 2 sessions
        let advice = m.advise("d", Risk::Safe, GIB / 2);
        assert!(advice.hot && advice.preselect);
        assert!(advice.reason.starts_with("often large"), "{}", advice.reason);
    }

    #[test]
    fn test_regrowth_rate_after_clean() {
        let mut m = Memory::default();
        seen(&mut m, "t", 100 * MIB, T0);
        m.record("t", Decision::Cleaned { freed: 100 * MIB }, T0);
        // Two days later it is back to 800 MB: 400 MB/day.
        seen(&mut m, "t", 800 * MIB, T0 + 2 * DAY);
        let rate = m.entries["t"].regrow_per_day.unwrap();
        assert!((rate - 400.0 * MIB as f64).abs() < 1.0, "{rate}");
        assert!(m.entries["t"].is_hot());
        // Later scans without a new clean do not change the rate.
        seen(&mut m, "t", 5 * GIB, T0 + 3 * DAY);
        assert_eq!(m.entries["t"].regrow_per_day, Some(rate));
    }

    #[test]
    fn test_regrowth_ignores_rescan_right_after_clean() {
        let mut m = Memory::default();
        seen(&mut m, "t", GIB, T0);
        m.record("t", Decision::Cleaned { freed: GIB }, T0);
        seen(&mut m, "t", 10 * MIB, T0 + 60);
        assert_eq!(m.entries["t"].regrow_per_day, None);
    }

    #[test]
    fn test_samples_are_bounded() {
        let mut m = Memory::default();
        for i in 0..(MAX_SAMPLES as u64 + 10) {
            seen(&mut m, "s", i, T0 + i);
        }
        let e = &m.entries["s"];
        assert_eq!(e.samples.len(), MAX_SAMPLES);
        assert_eq!(e.samples[0].bytes, 10);
        assert_eq!(e.times_seen as usize, MAX_SAMPLES + 10);
    }

    #[test]
    fn test_prune_forgets_stale_uncleaned_entries_only() {
        let mut m = Memory::default();
        seen(&mut m, "stale", GIB, T0);
        seen(&mut m, "cleaned", GIB, T0);
        m.record("cleaned", Decision::Cleaned { freed: GIB }, T0);
        seen(&mut m, "fresh", GIB, T0 + FORGET_AFTER);
        m.prune(T0 + FORGET_AFTER + 1);
        let ids: Vec<&str> = m.entries.keys().map(String::as_str).collect();
        assert_eq!(ids, ["cleaned", "fresh"]);
    }

    #[test]
    fn test_pending_offer_expires() {
        let mut m = Memory::default();
        m.offer(vec!["a".into()], T0);
        assert_eq!(m.take_pending_offer(T0 + 60), ["a"]);
        assert!(m.take_pending_offer(T0 + 60).is_empty(), "taken once");
        m.offer(vec!["a".into()], T0);
        assert!(m.take_pending_offer(T0 + 2 * DAY).is_empty());
    }

    #[test]
    fn test_record_unknown_id_is_ignored() {
        let mut m = Memory::default();
        m.record("nope", Decision::Kept, T0);
        assert!(m.entries.is_empty());
    }

    #[test]
    fn test_save_load_roundtrip_and_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sub/memory.json");
        assert_eq!(Memory::load(&path).unwrap().runs, 0);

        let mut m = Memory::default();
        m.begin_run(T0);
        seen(&mut m, "cache:Npm", GIB, T0);
        m.record("cache:Npm", Decision::Cleaned { freed: GIB }, T0);
        m.save(&path).unwrap();

        let loaded = Memory::load(&path).unwrap();
        assert_eq!(loaded.runs, 1);
        let e = &loaded.entries["cache:Npm"];
        assert_eq!((e.times_cleaned, e.total_freed, e.target.clone()), (1, GIB, npm()));
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn test_corrupt_file_is_an_error_not_silently_reset() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("memory.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(Memory::load(&path).is_err());
    }
}
