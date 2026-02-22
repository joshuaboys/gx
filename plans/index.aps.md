# gx — Git Project Manager

| Field | Value |
|-------|-------|
| Status | Draft |
| Owner | @joshuaboys |
| Created | 2026-02-22 |

## Problem

Cloning and navigating git repositories requires too many steps. `git clone` dumps repos wherever you are, paths are long and inconsistent, and jumping between projects means remembering deep directory structures. Existing tools (gclone) solve the cloning problem but are written in Go and lack project navigation.

## Success Criteria

- [ ] `gx clone user/repo` clones to organized directory and cd's into it
- [ ] `gx <name>` jumps to any indexed project with tab completion
- [ ] `gx ls` lists all indexed projects
- [ ] Full URL support (HTTPS, SSH, git://, shorthand)
- [ ] Configurable directory structure (flat vs host-prefixed)
- [ ] Compiles to single binary via `bun build --compile`

## Constraints

- TypeScript + Bun only (no Go, no Node)
- Must work as oh-my-zsh custom plugin for shell integration
- Index and config stored in `~/.config/gx/`
- Shell `cd` requires zsh plugin wrapper (subprocess can't change parent shell dir)

## Modules

| Module | Purpose | Status | Dependencies |
|--------|---------|--------|--------------|
| [URL & Path](./modules/01-url.aps.md) | Parse git URLs and map to filesystem paths | Draft | — |
| [Clone](./modules/02-clone.aps.md) | Clone repos to organized directories | Draft | URL |
| [Index](./modules/03-index.aps.md) | Track projects and resolve names to paths | Draft | — |
| [CLI](./modules/04-cli.aps.md) | Subcommand routing and argument parsing | Draft | URL, Clone, Index |
| [Shell Plugin](./modules/05-shell.aps.md) | Zsh plugin for cd, completion, and shell integration | Draft | CLI |

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bun compile binary size | Large binary for simple CLI | Acceptable for v1, benchmark and optimize later |
| URL edge cases | Malformed URLs cause crashes | Port gclone's validation regex, comprehensive test suite |
| Index name collisions | Two repos with same basename | Last-write-wins for v1, warn on collision |

## Open Questions

- [x] Which language? — TypeScript/Bun
- [x] Directory structure? — Flat (user/repo) by default, configurable
- [x] Default host? — github.com
- [ ] Should `gx rebuild` run automatically on a schedule or only manually?

## Decisions

- **D-001:** Flat directory structure (user/repo) by default, `host` mode (host/user/repo) via config — *accepted*
- **D-002:** github.com as default host for shorthand `user/repo` — *accepted*
- **D-003:** Binary + zsh plugin architecture — *accepted*
- **D-004:** `gx <name>` as default action (jump to project) — *accepted*
