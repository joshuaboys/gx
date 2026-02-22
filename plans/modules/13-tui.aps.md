# Interactive TUI

| ID | Owner | Status |
|----|-------|--------|
| TUI | @joshuaboys | Future |

## Purpose

Upgrade the static ANSI dashboard (Module 10) into a full interactive terminal UI. Users can navigate projects with keyboard controls, view git details inline, and jump into projects directly from the interface — similar to lazygit but for multi-project management.

## In Scope

- `gx tui` or `gx dash --interactive` — launch interactive terminal UI
- Keyboard navigation: arrow keys to move between projects, Enter to jump
- Project list with real-time git status (branch, dirty count, ahead/behind)
- Group filtering: toggle visibility of Dirty/Ahead/Clean/Stale groups
- Search/filter projects by name within the TUI
- Detail panel: expanded git info for selected project (recent commits, changed files)
- Graceful terminal resize handling

## Out of Scope

- Mouse support
- Git operations from within the TUI (commit, push, pull — read-only)
- Multi-pane layouts or split views
- Configuration editing from within the TUI
- Custom keybinding configuration

## Interfaces

**Depends on:**

- Dashboard (10) — reuses `DashboardData` collection and grouping logic
- Tracking (09) — `lastVisited` for display and default sort order

**Exposes:**

- `tui` subcommand in CLI
- `--interactive` flag on `dash` subcommand
- Interactive project selection returning selected project path

## Constraints

- Must work in standard terminal emulators (xterm-256color compatible)
- Must clean up terminal state on exit (no broken terminal after Ctrl+C)
- Should use a Bun-compatible terminal UI library or raw ANSI escape sequences
- Must not block on slow git operations — show loading state, update progressively
- Exit via `q`, `Ctrl+C`, or `Esc` must always work

## Ready Checklist

Change status to **Ready** when:

- [ ] Terminal UI library or raw approach chosen
- [ ] Keyboard navigation and keybinding scheme designed
- [ ] Layout and panel structure defined
- [ ] Dashboard module (10) is Complete or near-Complete
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- this module is in Future status._
