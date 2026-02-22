# Dashboard

| ID | Owner | Status |
|----|-------|--------|
| DASH | @joshuaboys | Draft |

## Purpose

Provide a single-command overview of all indexed projects grouped by git status. `gx dash` prints a colored ANSI table showing which projects have uncommitted changes, unpushed commits, or are clean/stale, giving users immediate visibility into their entire workspace.

## In Scope

- `gx dash` — display all indexed projects grouped by status category
- Status groups: Dirty (uncommitted changes), Ahead (unpushed commits), Clean, Stale (no commits in >30 days)
- Per-project display: project name, branch name, dirty file count, ahead/behind counts, last commit age
- Colored ANSI output with group headers
- Parallel git status/log queries across all indexed projects (bounded concurrency)
- Graceful handling of missing or inaccessible project directories
- Configurable stale threshold via `gx config set staleThreshold <days>`

## Out of Scope

- Interactive navigation or keyboard input (v4 TUI module handles this)
- Filtering by group or project name (future enhancement)
- Watch mode or auto-refresh
- Remote fetch before status check (uses local state only)
- Custom grouping or sorting rules

## Interfaces

**Depends on:**

- Index (03) — `list()` to enumerate all projects and their paths
- Tracking (09) — `lastVisited` for displaying recency info alongside status

**Exposes:**

- `dashboard(): Promise<DashboardData>` — collect git status for all indexed projects
- `renderDashboard(data: DashboardData): string` — format as colored ANSI output
- `dash` subcommand in CLI

## Constraints

- Must complete within a reasonable time even with 50+ projects — use bounded parallel execution (e.g., 8 concurrent git processes)
- Must not modify any project state — read-only git queries only
- ANSI colors must degrade gracefully when output is piped (detect TTY, strip colors for non-TTY)
- Must handle projects where `.git` directory exists but is corrupted or in a detached state
- Stale threshold default: 30 days, configurable

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Git status collection strategy defined (which git commands, concurrency model)
- [ ] ANSI output format and grouping rules designed
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
