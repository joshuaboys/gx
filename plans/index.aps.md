# gx — Git Project Manager

| Field | Value |
|-------|-------|
| Status | In Progress |
| Owner | @joshuaboys |
| Created | 2026-02-22 |
| v1 Completed | 2026-02-23 |
| v2 Completed | 2026-02-23 |

## Problem

Cloning and navigating git repositories requires too many steps. `git clone` dumps repos wherever you are, paths are long and inconsistent, and jumping between projects means remembering deep directory structures. Existing tools (gclone) solve the cloning problem but are written in Go and lack project navigation.

## Success Criteria

### v1 — Core CLI (Complete)

- [x] `gx clone user/repo` clones to organized directory and cd's into it
- [x] `gx <name>` jumps to any indexed project with tab completion
- [x] `gx ls` lists all indexed projects
- [x] Full URL support (HTTPS, SSH, git://, shorthand)
- [x] Configurable directory structure (flat vs host-prefixed)
- [x] Compiles to single binary via `bun build --compile`

### v2 — Quality of Life (Complete)

- [x] `gx <name>` suggests fuzzy matches when exact match not found
- [x] `gx open <name>` opens project in preferred editor
- [x] `gx init` scaffolds AI agent configuration for a project

### v3 — Project Awareness (Draft)

- [ ] `gx recent` lists projects sorted by last visited
- [ ] `gx resume <name>` jumps to project and prints context (branch, dirty files, last commit)
- [ ] `gx dash` shows all projects grouped by git status (dirty, ahead, clean, stale)
- [ ] Dashboard output is colored ANSI with group headers
- [ ] Parallel git status queries complete within reasonable time for 50+ projects

### v4 — Ecosystem (Future)

- [ ] `.gx.json` hooks run on project entry with trust/allow safety
- [ ] `gx new <name> --template <tpl>` creates projects from template repos
- [ ] `gx tui` provides interactive terminal UI for project navigation
- [ ] `gx tutorial` walks new users through features interactively

### v5 — Adoption & Reliability (Draft)

- [ ] One-command install + GitHub release binaries for low-friction onboarding
- [ ] Shell portability beyond zsh (bash + fish support)
- [ ] `gx doctor` and index diagnostics/stats for trust at scale

## Constraints

- TypeScript + Bun only (no Go, no Node)
- Must work as oh-my-zsh custom plugin for shell integration
- Index and config stored in `~/.config/gx/`
- Shell `cd` requires zsh plugin wrapper (subprocess can't change parent shell dir)

## Modules

| Module | Purpose | Status | Version | Dependencies |
|--------|---------|--------|---------|--------------|
| [URL & Path](./modules/01-url.aps.md) | Parse git URLs and map to filesystem paths | Complete | v1 | — |
| [Clone](./modules/02-clone.aps.md) | Clone repos to organized directories | Complete | v1 | URL |
| [Index](./modules/03-index.aps.md) | Track projects and resolve names to paths | Complete | v1 | — |
| [CLI](./modules/04-cli.aps.md) | Subcommand routing and argument parsing | Complete | v1 | URL, Clone, Index |
| [Shell Plugin](./modules/05-shell.aps.md) | Zsh plugin for cd, completion, and shell integration | Complete | v1 | CLI |
| [Fuzzy Matching](./modules/06-fuzzy.aps.md) | Suggest closest matches when exact project name not found | Complete | v2 | Index |
| [Editor Integration](./modules/07-editor.aps.md) | Open projects in preferred editor | Complete | v2 | Index, CLI |
| [Agent Scaffolding](./modules/08-agent.aps.md) | Scaffold AI agent configuration for projects | Complete | v2 | CLI |
| [Tracking](./modules/09-tracking.aps.md) | Record project visits and enable recency-based navigation | Draft | v3 | Index |
| [Dashboard](./modules/10-dashboard.aps.md) | Colored ANSI overview of all projects grouped by git status | Draft | v3 | Index, Tracking |
| [Hooks](./modules/11-hooks.aps.md) | Per-project onEnter commands with trust/allow safety | Future | v4 | Shell Plugin |
| [Templates](./modules/12-templates.aps.md) | Create new projects from template repositories | Future | v4 | Clone, Index |
| [Interactive TUI](./modules/13-tui.aps.md) | Keyboard-navigable interactive terminal dashboard | Future | v4 | Dashboard, Tracking |
| [Tutorial](./modules/14-tutorial.aps.md) | Interactive walkthrough teaching gx features | Future | v4 | All stable modules |
| [Distribution & Install UX](./modules/15-distribution.aps.md) | Release binaries, installer, and doctor flow for easy adoption | Draft | v5 | CLI |
| [Shell Portability](./modules/16-shell-portability.aps.md) | Bash/fish integration and completion parity | Draft | v5 | Shell Plugin, CLI |
| [Index Reliability & Observability](./modules/17-index-observability.aps.md) | Index diagnostics, stats, and scan telemetry | Draft | v5 | Index, Tracking |

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bun compile binary size | Large binary for simple CLI | Acceptable for v1, benchmark and optimize later |
| URL edge cases | Malformed URLs cause crashes | Port gclone's validation regex, comprehensive test suite |
| Index name collisions | Two repos with same basename | Last-write-wins for v1, warn on collision |
| Fuzzy match false positives | Auto-jump to wrong project on weak match | Require high similarity threshold for auto-jump, confirm otherwise |
| Editor detection fragility | `$EDITOR` not set or points to unknown binary | Maintain known-editor list, fall back to sensible default |
| Template staleness | Scaffolded CLAUDE.md becomes outdated as conventions change | Keep templates minimal and project-type-specific, easy to regenerate |
| Dashboard performance at scale | Git status across 100+ repos could be slow | Bounded concurrency (8 parallel), timeout per repo, cache recent results |
| Index schema migration | Adding `lastVisited` field to existing entries | Make field optional, null = never visited, no migration step required |
| Hook security surface | Malicious `.gx.json` could run arbitrary commands | direnv-style trust/allow — hooks never run until user explicitly trusts project |
| TUI terminal compatibility | Escape sequences differ across terminals | Target xterm-256color, degrade gracefully, always provide non-interactive fallback |

## Open Questions

- [x] Which language? — TypeScript/Bun
- [x] Directory structure? — Flat (user/repo) by default, configurable
- [x] Default host? — github.com
- [x] Should `gx rebuild` run automatically on a schedule or only manually? — Manual only for v1
- [ ] Which fuzzy matching algorithm? (Levenshtein, Jaro-Winkler, or substring containment)
- [ ] Should `gx open` support opening specific files within a project?
- [ ] Should `gx init` overwrite existing `.claude/CLAUDE.md` or merge?
- [ ] Should `gx recent` show a configurable default count or always show all?
- [ ] What git commands should `gx resume` run for context? (`git status --short`, `git log -1 --oneline`, `git branch --show-current`)
- [ ] Should `gx dash` fetch remotes before reporting ahead/behind, or use local state only?
- [ ] What stale threshold default? (30 days proposed)
- [ ] Should hook trust be per-project or per-file-hash (re-trust on `.gx.json` change)?
- [ ] Which TUI library for interactive mode? (ink, blessed, raw ANSI, or Bun-native)

## Decisions

- **D-001:** Flat directory structure (user/repo) by default, `host` mode (host/user/repo) via config — *accepted*
- **D-002:** github.com as default host for shorthand `user/repo` — *accepted*
- **D-003:** Binary + zsh plugin architecture — *accepted*
- **D-004:** `gx <name>` as default action (jump to project) — *accepted*
- **D-005:** Dashboard uses local git state only — no remote fetch on `gx dash` (too slow for multi-project scan) — *proposed*
- **D-006:** `lastVisited` is an optional field on IndexEntry — backward compatible, no migration needed — *proposed*
- **D-007:** v3 before v4 — tracking and dashboard provide immediate value and inform TUI design — *proposed*
