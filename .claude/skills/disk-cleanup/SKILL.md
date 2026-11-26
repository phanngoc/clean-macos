---
name: disk-cleanup
description: Free disk space on this machine with the repo's cache-cleaner CLI. It scans caches, Docker, WSL, build output of idle projects and large stale folders, suggests what to delete (pre-ticked from what it learned about the user's past choices), and cleans only the ids the user picks. Use for "ổ đầy", "dọn ổ C", "giải phóng dung lượng", "dọn cache", "xóa node_modules cũ", "cái gì chiếm nhiều dung lượng", "free up disk space", "clean caches", or "what is eating my disk".
---

# disk-cleanup

Drive `cache-cleaner-cli` (in `cache-cleaner-app/src-tauri`) through its JSON interface. The CLI
keeps a self-learning memory at `~/.cache-cleaner/memory.json`: sizes of every place per scan
(hot spots = large across sessions or refilling fast) and what the user cleaned or kept
(habits → what gets pre-ticked next time). Your job is to show the suggestions, let the user
decide, and pass exactly their decision back so the memory learns the right thing.

## 1. Get the binary

```bash
cd cache-cleaner-app/src-tauri
ls target/release/cache-cleaner-cli* 2>/dev/null \
  || cargo build --release --no-default-features --bin cache-cleaner-cli
```

Use `target/release/cache-cleaner-cli` (`.exe` on Windows) below as `$CLI`.

## 2. Scan (read-only)

```bash
$CLI memory --json                 # what was learned before: hot spots, habits, last run
$CLI suggest --json                # full scan, ~30 s to a few minutes; deletes nothing
$CLI suggest --quick --json        # only re-measure remembered places (use when memory has a recent run)
```

Useful flags: `--min-mb 100` (hide small items), `--root PATH` / `--root wsl:/path` (extra
project folders), `--no-wsl`, `--no-artifacts`, `--idle-days 30`.

Each suggestion has: `id`, `label`, `location`, `size_bytes`/`size`, `risk`
(`safe` | `review` | `danger`), `note` (what deleting means), `preselect` + `reason` (from memory),
`hot`, `covered_by` (nested in another suggestion; selecting the parent already deletes it),
and `history`. `total_bytes` and `preselected_bytes` already count nested items once.

## 3. Ask the user

Present a compact table, largest first: size, label, risk, and the reason. Mark pre-ticked rows
and hot spots (🔥). Then ask which ids to delete. Recommend the pre-ticked set, but:

- Never include a `danger` item (Docker volumes, device backups, Xcode archives) unless the user
  names it explicitly after you explain the `note`.
- For `review` items, say what is lost (`note`) before recommending them.
- Do not offer both a parent and its `covered_by` children; the parent is enough.
- Do not decide for the user. "Dọn hết cái được gợi ý" counts as consent to the pre-ticked set.

## 4. Clean exactly the chosen ids

```bash
$CLI suggest --select <id> --select <id> --dry-run --json   # optional preview
$CLI suggest --select <id> --select <id> --json             # deletes
```

Pass ids verbatim (quote them: they contain `:` and paths). `--select` re-measures each id
first and refuses ids it never saw (exit code 2), so run step 2 before it. Everything shown in
the last `--json` listing (within 24 h) that was not selected is remembered as *kept*: that is
how the memory learns. So only run `--select` after the user has actually chosen.
Do not use `--yes` for the user; it cleans the pre-ticked set without asking.

## 5. Report

From `results`: freed bytes per item, failures with their `message` (locked files and
admin-only locations are skipped, not fatal). Then run `$CLI memory --json` and tell the user
what will be pre-ticked next time and which places are hot spots, so they can correct it:

```bash
$CLI memory forget <id>     # stop remembering one place
$CLI memory reset --yes     # forget everything (only if the user asks)
```

For an interactive session in their own terminal, the user can run `$CLI` with no arguments:
a checkbox list (space to tick, enter to delete).
