# gx v3 & v4 Roadmap

## Overview

This document covers the design and phasing rationale for gx versions 3 and 4. v1 (core CLI) and v2 (fuzzy matching, editor integration, agent scaffolding) established the foundation. v3 adds project awareness — knowing *when* you last worked on something and *what state* it's in. v4 builds on that awareness to add automation, scaffolding, and interactive navigation.

## v3 Scope: Project Awareness

v3 introduces two modules that transform gx from a navigation tool into a workspace awareness tool.

### Module 09: Project Tracking (TRK)

**Problem:** Users juggle many projects but have no memory of which ones they were working on recently. Returning to a project means remembering its name, then running `git status` and `git log` to re-establish context.

**Solution:**

- Add a `lastVisited` timestamp to each index entry, updated automatically on every `gx <name>` or `gx clone`
- `gx recent` lists projects sorted by most recently visited (supports `-n <N>` for limiting output)
- `gx resume <name>` jumps to the project and prints context: current branch, dirty file count, and last commit summary
- Internal `gx touch <name>` command allows the shell plugin to record visits without full resolution

**Index schema change:**

```json
{
  "projects": {
    "gx": {
      "path": "/home/user/Projects/src/joshuaboys/gx",
      "url": "https://github.com/joshuaboys/gx",
      "clonedAt": "2026-02-22T00:00:00Z",
      "lastVisited": "2026-02-23T14:30:00Z"
    }
  }
}
```

The `lastVisited` field is optional. Existing entries without it are treated as "never visited" and sort to the bottom of `gx recent`. No migration step is needed.

**Key design decisions:**
- Recency only, not frecency. Frequency-weighted scoring adds complexity with minimal benefit for the common use case (returning to what you were just working on).
- `gx touch` is deliberately fast — no stdout, minimal JSON read-write. It runs on every navigation event via the shell plugin, so it must not add perceptible latency.
- `gx resume` gathers context via `git branch --show-current`, `git status --short`, and `git log -1 --oneline`. These are fast, local-only commands.

### Module 10: Dashboard (DASH)

**Problem:** With dozens of indexed projects, users lose track of which repos have uncommitted work, unpushed commits, or have gone stale. There is no single view across the workspace.

**Solution:**

- `gx dash` prints a colored ANSI table showing all projects grouped by status
- Four groups: **Dirty** (uncommitted changes), **Ahead** (unpushed commits), **Clean** (fully synced), **Stale** (no commits in >30 days)
- Each row shows: project name, branch, dirty file count, ahead/behind counts, last commit age

**Example output sketch:**

```
 Dirty (3)
  gx                main    3 files   2m ago
  dotfiles          master  1 file    5h ago
  website           feat/v2 12 files  1d ago

 Ahead (1)
  api-server        main    +3        3h ago

 Clean (8)
  cli-tools         main              1d ago
  ...

 Stale (2)
  old-experiment    main              45d ago
  archived-thing    master            120d ago
```

**Performance strategy:**
- Bounded concurrency: spawn up to 8 `git` processes in parallel
- Per-repo timeout: skip repos that don't respond within 5 seconds
- No remote fetch — uses local state only (fetching remotes for 50+ repos would take too long and require network)
- Graceful degradation: missing or corrupted repos show as "unavailable" rather than crashing

**TTY detection:** When output is piped (non-TTY), ANSI color codes are stripped automatically. This allows `gx dash | grep dirty` workflows.

## v4 Scope: Ecosystem

v4 modules are currently in Future status. They are designed but not yet scoped into work items. Each builds on v3's awareness layer.

### Module 11: Project Hooks (HOOK)

**Problem:** Different projects need different environments — Node version managers, Python virtualenvs, environment variables, secrets. Users manually run setup commands every time they switch projects.

**Solution:**

A `.gx.json` file in the project root defines `onEnter` hooks:

```json
{
  "onEnter": [
    "fnm use",
    "source .venv/bin/activate"
  ]
}
```

**Security model:** Hooks never run without explicit trust. On first encounter, gx warns the user and requires `gx trust <name>` before hooks will execute. This follows the direnv trust model — simple, proven, and avoids accidental execution of malicious commands from cloned repos.

**agent-vault integration:** For projects that need secrets injected into the environment, hooks can reference agent-vault entries. This is optional — hooks work without agent-vault installed.

### Module 12: Project Templates (TPL)

**Problem:** Creating new projects involves cloning a template repo, deleting `.git`, running `git init`, and manually indexing. This is tedious and error-prone.

**Solution:**

- `gx new <name> --template <template>` clones a template repo, removes its git history, reinitializes, and auto-indexes
- `gx template add/list/rm` manages a local template registry in `~/.config/gx/templates.json`
- Templates are plain git repos — no special format, no variable substitution (keeping it simple)

### Module 13: Interactive TUI (TUI)

**Problem:** The static dashboard (v3) shows status but requires re-running the command to see updates. Users want to browse projects interactively and jump directly from the overview.

**Solution:**

- `gx tui` or `gx dash --interactive` launches a keyboard-navigable terminal UI
- Arrow keys navigate, Enter jumps to selected project, `/` searches
- Reuses all data collection from Module 10 (Dashboard)
- This is why v3 ships Dashboard as static ANSI first — it validates the data model and grouping logic before committing to a TUI framework

### Module 14: Tutorial (TUTR)

**Problem:** New users face a learning curve discovering gx features. Documentation is passive; an interactive tutorial is more effective.

**Solution:**

- `gx tutorial` walks users through core features step by step
- Each step explains, runs a real command, shows output, and waits for the user to proceed
- Covers v1-v3 features; v4 features are added to the tutorial when they ship

## Phasing Rationale

**Why v3 before v4?**

1. **Immediate value with low risk.** Tracking and dashboard are additive features that don't change existing behavior. They extend the index format with an optional field and add read-only git queries. No existing workflows break.

2. **Dashboard informs TUI design.** Building the static dashboard first lets us validate grouping logic, git status collection, and performance characteristics before committing to a TUI framework. If the static version reveals that certain git queries are too slow or certain groupings are not useful, we learn that cheaply.

3. **Tracking enables better hooks.** When v4 hooks land, `lastVisited` data can inform which projects have active hooks (recently visited = likely to trigger). This avoids unnecessary hook validation for dormant projects.

4. **Templates and hooks are independent.** v4 modules have minimal interdependence (except TUI depending on Dashboard). They can be built in any order once v3 is stable.

**Why not ship hooks in v3?**

Hooks introduce a security surface (arbitrary command execution) that requires careful design. The trust/allow model needs user testing before committing. v3 ships safely without it. Additionally, hooks depend on the shell plugin changes, which are more invasive than the index extension that tracking requires.

**Why is TUI in v4 instead of v3?**

A terminal UI framework is a significant dependency and maintenance burden. Shipping the static dashboard first proves the value proposition. If users don't use `gx dash`, investing in an interactive version would be wasted effort.

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Recency vs frecency | Recency only | Simpler, covers primary use case, frequency adds little value for project navigation |
| Remote fetch in dashboard | Local state only | Fetching remotes for N repos is too slow; users can `git fetch` individually |
| Index schema migration | Optional field, no migration | Backward compatible; old entries work without `lastVisited` |
| Dashboard concurrency | 8 parallel git processes | Balances speed vs system load; configurable in future if needed |
| Stale threshold | 30 days default, configurable | Reasonable heuristic; `gx config set staleThreshold <days>` for customization |
| Hook security | direnv-style trust/allow | Proven model, simple UX, prevents accidental execution |
| TUI phasing | Static ANSI first, interactive later | Validates data model cheaply before committing to framework |

## Dependencies

```
v1 (Complete)          v2 (Draft)           v3 (Draft)          v4 (Future)
----------------       ---------------      ---------------     ----------------
01-URL                 06-Fuzzy             09-Tracking         11-Hooks
02-Clone               07-Editor            10-Dashboard        12-Templates
03-Index               08-Agent                                 13-TUI
04-CLI                                                          14-Tutorial
05-Shell Plugin

                       06 <- 03
                       07 <- 03, 04
                       08 <- 04
                       09 <- 03
                       10 <- 03, 09
                       11 <- 05
                       12 <- 02, 03
                       13 <- 09, 10
                       14 <- all stable
```

## Next Steps

1. Finalize v2 modules (fuzzy, editor, agent) — these are lower priority but simple
2. Design `IndexEntry` schema extension for `lastVisited` (backward compatible)
3. Prototype `gx recent` and `gx resume` commands
4. Prototype `gx dash` with a small set of repos to validate grouping and performance
5. Move Tracking (09) and Dashboard (10) to Ready status with defined work items
