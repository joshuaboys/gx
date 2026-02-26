# gx status — Repo Health Dashboard

| Field | Value |
|-------|-------|
| Created | 2026-02-26 |
| Status | Approved |
| Module | DASH (10-dashboard) |
| Version | v3 |

## Problem

With 30+ repos in `~/src/`, there's no quick way to know which repos have
uncommitted changes, unpushed commits, stale stashes, or lingering worktrees.
The manual equivalent requires running 5-6 git commands per repo — tedious and
easy to forget before a cleanup or re-clone.

## Solution

`gx status` — a single command that scans all indexed projects and prints a
full-color health report with four sections: clean repos, dirty repos, stash
inventory, and worktree inventory.

## Command Interface

```
gx status              # full color report
gx status --dirty      # only dirty repos
gx status --json       # machine-readable JSON, no color
```

## Output Format

```
 Clean (3)
 Repo              Branch              Owner
 anvil             main                EddaCraft
 code-review       main                joshuaboys
 gx                chore/bump-v0.2.0   joshuaboys

 Dirty (4)
 Repo              Branch    Owner         Issues
 claw              main      joshuaboys    1 unpushed
 wintermute        main      joshuaboys    23 uncommitted, 1 stash (stale)
 eddacode          main      joshuaboys    1 uncommitted, 4 stashes (stale)
 dev-env           main      joshuaboys    1 uncommitted, 1 stash

 Stashes (3 repos, 6 total, 5 stale)
 Repo              #   Date         Description
 dev-env           0   Feb 15       WIP on main: dotfile sync
 eddacode          0   Feb 03       WIP on main: plugin manifest fix
 eddacode          1   Feb 03       WIP on main: esbuild external fix
 eddacode          2   Jan 31       WIP on main: memory/ralph bug fixes     ← stale
 eddacode          3   Feb 02       WIP on bug-fixing: memory watcher leak  ← stale
 wintermute        0   Feb 09       WIP on main: restore codex/gemini MCP   ← stale

 Worktrees (1 repo, 2 extra)
 Repo              Path                    Branch
 anvil             ~/src/anvil.PRs         test/tutorial-continuation-coverage
 anvil             ~/src/anvil.PRs5        copilot/fix-flaky-tutorial-keyboard-tests
```

## Color Scheme

| Element | Color |
|---------|-------|
| Section headers | Bold white |
| Clean repos | Green |
| Dirty repos | Yellow |
| Stale stashes | Red + `← stale` marker |
| Worktrees | Cyan |
| Counts in headers | Bold |

TTY detection: strip ANSI codes when stdout is not a TTY (piped output).

## Architecture

### Files

- `src/commands/status.ts` — command handler, git data collection, rendering
- `src/lib/color.ts` — ANSI color helpers with TTY detection (no dependencies)

### Git Checks Per Repo

Six git commands, all read-only:

1. `git branch --show-current`
2. `git remote get-url origin`
3. `git status --porcelain`
4. `git log --oneline @{upstream}..HEAD` (suppress stderr for no-upstream)
5. `git stash list --format=%gd|%ci|%gs`
6. `git worktree list --porcelain`

All six run concurrently per repo via `Bun.spawn`. Repos are also checked
concurrently via `Promise.all` with bounded concurrency (limit 8).

### Owner Parsing

Reuse `parseRepoUrl()` from `src/lib/url.ts` to extract owner from the origin
remote URL.

### Staleness

A stash is "stale" if its parent commit date is >14 days before the current
date. Hardcoded threshold for v1 — configurable `stalenessDays` can be added to
config later.

### Wire-up

- Add `status` case to `src/index.ts` switch statement
- Add `gx status` to help text
- Add `status` to shell completion list in `src/commands/shell-init.ts`

## Design Decisions

- **`gx status` not `gx dash`** — mirrors `git status`, more intuitive naming.
  The existing DASH module in APS is renamed/evolved to match.
- **No Tracking dependency** — status works without visit history (module 09).
  Tracking can enrich it later but is not a prerequisite.
- **Full report by default** — show all four sections. `--dirty` filters down.
  Users want "show me everything" as the default action.
- **Local state only** — no `git fetch` before checks. Too slow for multi-repo
  scans. D-005 from the index plan already established this.
- **No external dependencies for color** — raw ANSI escape codes via a small
  helper module. gx has zero runtime deps and should stay that way.

## Constraints

- Read-only — must not modify any project state
- Must handle missing directories, corrupted `.git`, detached HEAD gracefully
- Must complete in reasonable time for 50+ projects (bounded concurrency)
- Binary size impact should be negligible (no new dependencies)
