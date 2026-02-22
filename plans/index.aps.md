# gx — Git Project Manager

| Field | Value |
|-------|-------|
| Status | In Progress |
| Owner | @joshuaboys |
| Created | 2026-02-22 |
| v1 Completed | 2026-02-23 |

## Problem

Cloning and navigating git repositories requires too many steps. `git clone` dumps repos wherever you are, paths are long and inconsistent, and jumping between projects means remembering deep directory structures. Existing tools (gclone) solve the cloning problem but are written in Go and lack project navigation.

## Success Criteria

- [x] `gx clone user/repo` clones to organized directory and cd's into it
- [x] `gx <name>` jumps to any indexed project with tab completion
- [x] `gx ls` lists all indexed projects
- [x] Full URL support (HTTPS, SSH, git://, shorthand)
- [x] Configurable directory structure (flat vs host-prefixed)
- [x] Compiles to single binary via `bun build --compile`
- [ ] `gx <name>` suggests fuzzy matches when exact match not found
- [ ] `gx open <name>` opens project in preferred editor
- [ ] `gx init` scaffolds AI agent configuration for a project

## Constraints

- TypeScript + Bun only (no Go, no Node)
- Must work as oh-my-zsh custom plugin for shell integration
- Index and config stored in `~/.config/gx/`
- Shell `cd` requires zsh plugin wrapper (subprocess can't change parent shell dir)

## Modules

| Module | Purpose | Status | Dependencies |
|--------|---------|--------|--------------|
| [URL & Path](./modules/01-url.aps.md) | Parse git URLs and map to filesystem paths | Complete | — |
| [Clone](./modules/02-clone.aps.md) | Clone repos to organized directories | Complete | URL |
| [Index](./modules/03-index.aps.md) | Track projects and resolve names to paths | Complete | — |
| [CLI](./modules/04-cli.aps.md) | Subcommand routing and argument parsing | Complete | URL, Clone, Index |
| [Shell Plugin](./modules/05-shell.aps.md) | Zsh plugin for cd, completion, and shell integration | Complete | CLI |
| [Fuzzy Matching](./modules/06-fuzzy.aps.md) | Suggest closest matches when exact project name not found | Draft | Index |
| [Editor Integration](./modules/07-editor.aps.md) | Open projects in preferred editor | Draft | Index, CLI |
| [Agent Scaffolding](./modules/08-agent.aps.md) | Scaffold AI agent configuration for projects | Draft | CLI |

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bun compile binary size | Large binary for simple CLI | Acceptable for v1, benchmark and optimize later |
| URL edge cases | Malformed URLs cause crashes | Port gclone's validation regex, comprehensive test suite |
| Index name collisions | Two repos with same basename | Last-write-wins for v1, warn on collision |
| Fuzzy match false positives | Auto-jump to wrong project on weak match | Require high similarity threshold for auto-jump, confirm otherwise |
| Editor detection fragility | `$EDITOR` not set or points to unknown binary | Maintain known-editor list, fall back to sensible default |
| Template staleness | Scaffolded CLAUDE.md becomes outdated as conventions change | Keep templates minimal and project-type-specific, easy to regenerate |

## Open Questions

- [x] Which language? — TypeScript/Bun
- [x] Directory structure? — Flat (user/repo) by default, configurable
- [x] Default host? — github.com
- [x] Should `gx rebuild` run automatically on a schedule or only manually? — Manual only for v1
- [ ] Which fuzzy matching algorithm? (Levenshtein, Jaro-Winkler, or substring containment)
- [ ] Should `gx open` support opening specific files within a project?
- [ ] Should `gx init` overwrite existing `.claude/CLAUDE.md` or merge?

## Decisions

- **D-001:** Flat directory structure (user/repo) by default, `host` mode (host/user/repo) via config — *accepted*
- **D-002:** github.com as default host for shorthand `user/repo` — *accepted*
- **D-003:** Binary + zsh plugin architecture — *accepted*
- **D-004:** `gx <name>` as default action (jump to project) — *accepted*
