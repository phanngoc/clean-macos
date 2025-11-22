#!/usr/bin/env bash
# empty-trash.sh
# Fast and flexible Trash emptier for macOS (works on M1/M2/M3 Apple Silicon)
# Part of the clean-macos suite
#
# Modes:
#   - swap (default): Atomic directory swap for instant space reclaim + background deletion
#   - parallel: Multi-threaded deletion with configurable worker count
#
# Usage: ./empty-trash.sh [--mode parallel|swap] [--dry-run] [--include-volumes] [--confirm] [--jobs N]

set -euo pipefail

# Configuration
MODE="swap"            # default mode: "swap" or "parallel"
DRY_RUN=false
INCLUDE_VOLUMES=false
CONFIRM=false
PARALLEL_JOBS=4        # number of parallel rm workers for parallel mode
UID_NUM="$(id -u)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

function usage() {
  cat <<EOF
${BLUE}empty-trash.sh${NC} - Fast macOS Trash Cleaner

${GREEN}Usage:${NC}
  $0 [OPTIONS]

${GREEN}Options:${NC}
  --mode parallel    Use parallel deletion (find | xargs -P)
  --mode swap        Use atomic swap: replace trash with empty dir, then delete in background (default)
  --dry-run          Show what would be removed (no deletion)
  --include-volumes  Also clean external volumes' trashes under /Volumes/*/.Trashes/<UID>
  --confirm          Skip interactive confirmation (use with care)
  --jobs N           Number of parallel jobs for parallel mode (default: 4)
  -h, --help         Show this help message

${GREEN}Examples:${NC}
  $0 --dry-run                      # Preview what will be deleted
  $0                                # Empty trash using swap mode (default)
  $0 --mode parallel --jobs 8       # Use parallel deletion with 8 workers
  $0 --include-volumes              # Include external drive trash folders
  $0 --confirm                      # Skip confirmation prompt

${YELLOW}Safety:${NC}
  - Requires "YES" confirmation by default (use --confirm to skip)
  - Use --dry-run to preview before deleting
  - Files are permanently deleted (no undo)
  - External volumes require --include-volumes flag

${BLUE}Mode Comparison:${NC}
  swap     → Instant space reclaim, background deletion (recommended)
  parallel → Direct multi-threaded deletion, no background jobs

EOF
  exit 0
}

# Parse command-line arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --mode) MODE="$2"; shift 2 ;;
    --mode=*) MODE="${1#*=}"; shift ;;
    --dry-run) DRY_RUN=true; shift ;;
    --include-volumes) INCLUDE_VOLUMES=true; shift ;;
    --confirm) CONFIRM=true; shift ;;
    --jobs) PARALLEL_JOBS="$2"; shift 2 ;;
    --jobs=*) PARALLEL_JOBS="${1#*=}"; shift ;;
    -h|--help) usage ;;
    *) echo -e "${RED}Unknown argument: $1${NC}"; usage ;;
  esac
done

# Validate mode
if [[ "$MODE" != "swap" && "$MODE" != "parallel" ]]; then
  echo -e "${RED}Error: Invalid mode '$MODE'. Must be 'swap' or 'parallel'${NC}"
  exit 1
fi

# Validate parallel jobs count
if ! [[ "$PARALLEL_JOBS" =~ ^[0-9]+$ ]] || [[ "$PARALLEL_JOBS" -lt 1 ]]; then
  echo -e "${RED}Error: --jobs must be a positive integer${NC}"
  exit 1
fi

# Find all trash directories
function find_trash_dirs() {
  local arr=()

  # Current user's main Trash
  arr+=("$HOME/.Trash")

  if $INCLUDE_VOLUMES; then
    # Look for per-volume trashes for this UID
    for vol in /Volumes/*; do
      [[ -d "$vol" ]] || continue

      # Standard path: /Volumes/<name>/.Trashes/<uid>
      tdir="$vol/.Trashes/$UID_NUM"
      if [[ -d "$tdir" ]]; then
        arr+=("$tdir")
      fi

      # Some volumes use top-level .Trash
      tdir2="$vol/.Trash"
      if [[ -d "$tdir2" ]]; then
        arr+=("$tdir2")
      fi
    done
  fi

  # Remove duplicates and output
  printf "%s\n" "${arr[@]}" | awk '!seen[$0]++'
}

# Collect trash directories
TRASH_DIRS=()
while IFS= read -r d; do
  TRASH_DIRS+=("$d")
done < <(find_trash_dirs)

if [[ ${#TRASH_DIRS[@]} -eq 0 ]]; then
  echo -e "${YELLOW}No Trash directories found.${NC}"
  exit 0
fi

# Display trash directories
echo -e "${BLUE}Trash Directories:${NC}"
for d in "${TRASH_DIRS[@]}"; do
  echo "  $d"
done
echo

# Preview sizes
function preview_sizes() {
  echo -e "${BLUE}Current Trash Sizes:${NC}"
  for d in "${TRASH_DIRS[@]}"; do
    if [[ -d "$d" ]]; then
      size=$(du -sh "$d" 2>/dev/null | cut -f1 || echo "unknown")
      echo "  $d: $size"
    else
      echo "  $d: (not present)"
    fi
  done
}

preview_sizes
echo

# Dry run mode
if $DRY_RUN; then
  echo -e "${YELLOW}DRY RUN MODE: Listing contents (no deletion will occur)${NC}"
  echo
  for d in "${TRASH_DIRS[@]}"; do
    if [[ -d "$d" ]]; then
      echo -e "${BLUE}Contents of $d:${NC}"
      find "$d" -maxdepth 2 -mindepth 1 2>/dev/null | head -20 | sed 's/^/  /' || echo "  (empty or inaccessible)"
      count=$(find "$d" -mindepth 1 2>/dev/null | wc -l | tr -d ' ')
      echo "  Total items: $count"
      echo
    fi
  done
  echo -e "${GREEN}Dry run complete. No files were deleted.${NC}"
  exit 0
fi

# Confirmation prompt
if ! $CONFIRM; then
  echo -e "${YELLOW}WARNING: This will permanently delete all trash contents!${NC}"
  echo -e "Mode: ${BLUE}$MODE${NC}"
  if [[ "$MODE" == "parallel" ]]; then
    echo -e "Parallel jobs: ${BLUE}$PARALLEL_JOBS${NC}"
  fi
  echo
  read -rp "Type YES to continue: " ans
  if [[ "$ans" != "YES" ]]; then
    echo -e "${RED}Cancelled.${NC}"
    exit 0
  fi
fi

# Execute deletion based on mode
if [[ "$MODE" == "parallel" ]]; then
  echo -e "${BLUE}Using parallel deletion mode with $PARALLEL_JOBS jobs...${NC}"
  echo

  for d in "${TRASH_DIRS[@]}"; do
    if [[ -d "$d" ]]; then
      echo -e "Deleting contents of ${YELLOW}$d${NC} ..."

      # Delete contents in parallel while preserving the directory
      # Use find -print0 and xargs -0 -P for safe parallel deletion
      find "$d" -mindepth 1 -print0 2>/dev/null | \
        xargs -0 -n1 -P "$PARALLEL_JOBS" -I{} sh -c 'rm -rf -- "$1" 2>/dev/null' _ "{}" || true

      echo -e "${GREEN}✓${NC} Done: $d"
    else
      echo -e "${YELLOW}⊘${NC} Skip (not found): $d"
    fi
  done

  echo
  echo -e "${GREEN}Parallel deletion completed successfully!${NC}"

elif [[ "$MODE" == "swap" ]]; then
  echo -e "${BLUE}Using swap mode (instant space reclaim + background deletion)...${NC}"
  echo

  for d in "${TRASH_DIRS[@]}"; do
    if [[ -d "$d" ]]; then
      parent="$(dirname "$d")"
      base="$(basename "$d")"
      tmpdir="$(mktemp -d "$parent/.empty.$base.XXXXX")"
      oldbackup="${d}.old.$(date +%s)"

      echo -e "Swapping: ${YELLOW}$d${NC} → ${YELLOW}$oldbackup${NC}"

      # Atomic swap: move current trash out, move empty dir in
      if mv -f "$d" "$oldbackup" 2>/dev/null; then
        if mv -f "$tmpdir" "$d" 2>/dev/null; then
          # Start background deletion
          nohup bash -c "rm -rf -- \"$oldbackup\"" >/dev/null 2>&1 &
          pid=$!
          echo -e "${GREEN}✓${NC} Space reclaimed. Background deletion started (PID: $pid)"
        else
          echo -e "${RED}✗${NC} Failed to create empty directory. Restoring..."
          mv -f "$oldbackup" "$d" 2>/dev/null || echo "  Recovery failed!"
          rm -rf "$tmpdir" 2>/dev/null
        fi
      else
        echo -e "${RED}✗${NC} Failed to swap directory. Cleaning up temp..."
        rm -rf "$tmpdir" 2>/dev/null
      fi
    else
      echo -e "${YELLOW}⊘${NC} Skip (not found): $d"
    fi
  done

  echo
  echo -e "${GREEN}Swap completed! Old trash directories are being removed in background.${NC}"
  echo -e "${YELLOW}Note: Background deletion continues even if you close this terminal.${NC}"
fi

# Final status
echo
preview_sizes
echo
echo -e "${GREEN}✓ Trash emptying complete!${NC}"
