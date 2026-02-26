# Status Dashboard

| ID | Owner | Status |
|----|-------|--------|
| DASH | @joshuaboys | Ready |

## Purpose

Provide a single-command overview of all indexed projects with git health
status. `gx status` prints a colored ANSI report with four sections: clean
repos, dirty repos, stash inventory, and worktree inventory.

## Design

See [2026-02-26-gx-status-design.md](../../docs/plans/2026-02-26-gx-status-design.md)
for the full design document.

## In Scope

- `gx status` — full color report (clean + dirty + stashes + worktrees)
- `gx status --dirty` — filter to dirty repos only
- `gx status --json` — machine-readable JSON output, no color
- Per-project: branch, owner (from remote), uncommitted count, unpushed count
- Stash inventory with dates and staleness detection (>14 days)
- Worktree inventory showing extra worktrees per repo
- Colored ANSI output with TTY detection (strip colors when piped)
- Parallel git queries with bounded concurrency (8)
- Graceful handling of missing/inaccessible directories

## Out of Scope

- Interactive navigation (v4 TUI module)
- Remote fetch before status check (local state only, per D-005)
- Visit tracking or recency info (module 09)
- Watch mode or auto-refresh
- Configurable staleness threshold (hardcoded 14 days for v1)

## Interfaces

**Depends on:**

- Index (03) — `list()` to enumerate all projects and their paths
- URL (01) — `parseRepoUrl()` to extract owner from origin remote

**Exposes:**

- `status(indexPath, config, flags)` — collect git health and render report
- `status` subcommand in CLI
- `StatusResult` type for JSON output

## Constraints

- Read-only — must not modify any project state
- Must complete in reasonable time for 50+ projects (bounded concurrency of 8)
- ANSI colors degrade when piped (TTY detection)
- Must handle corrupted `.git`, detached HEAD, missing directories
- Zero new runtime dependencies (raw ANSI escape codes)

## Work Items

### DASH-1: ANSI color helper module

| Field | Value |
|-------|-------|
| Status | Todo |
| Estimate | S |

Create `src/lib/color.ts` with:
- ANSI escape code helpers: `bold`, `green`, `yellow`, `red`, `cyan`, `dim`, `reset`
- `isTTY()` detection — strip colors when stdout is not a terminal
- No external dependencies

### DASH-2: Git data collection

| Field | Value |
|-------|-------|
| Status | Todo |
| Estimate | M |
| Depends | — |

Create core data collection in `src/commands/status.ts`:
- `RepoStatus` type: name, path, branch, owner, uncommitted count, unpushed
  count, stash list (with dates/descriptions), worktree list
- `collectRepoStatus(path)` — run 6 git commands concurrently via `Bun.spawn`,
  parse outputs
- `collectAllStatuses(index, config)` — iterate all indexed projects with
  bounded concurrency (8 parallel), handle missing dirs gracefully
- Owner extraction via `parseRepoUrl()` from `src/lib/url.ts`

### DASH-3: Report renderer

| Field | Value |
|-------|-------|
| Status | Todo |
| Estimate | M |
| Depends | DASH-1, DASH-2 |

Render functions in `src/commands/status.ts`:
- `renderStatus(results, flags)` — orchestrate four sections
- Clean table: repo, branch, owner (green)
- Dirty table: repo, branch, owner, issues summary (yellow)
- Stash table: repo, index, date, description, stale marker (red if stale)
- Worktree table: repo, path, branch (cyan)
- Column alignment via max-width padding
- Section headers with bold counts
- `--dirty` flag: skip clean table
- `--json` flag: output structured JSON, no ANSI

### DASH-4: CLI wire-up

| Field | Value |
|-------|-------|
| Status | Todo |
| Estimate | S |
| Depends | DASH-3 |

- Add `status` case to `src/index.ts` switch with flag parsing (`--dirty`,
  `--json`)
- Add `gx status` to help text
- Add `status` to completion list in `src/commands/shell-init.ts`

### DASH-5: Tests

| Field | Value |
|-------|-------|
| Status | Todo |
| Estimate | M |
| Depends | DASH-2, DASH-3 |

- `tests/commands/status.test.ts`
- Test `collectRepoStatus` with a temp git repo (clean, dirty, with stashes,
  with worktrees)
- Test `renderStatus` output for each section (color stripped for assertions)
- Test `--dirty` filtering
- Test `--json` output structure
- Test graceful handling of missing directory
- Test owner parsing from various remote URL formats
