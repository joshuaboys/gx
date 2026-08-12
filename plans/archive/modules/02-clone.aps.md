<!-- Archived: 2026-08-12 | Reason: All work items complete -->
<!-- Reopened 2026-08-03 for CLN-002 (one-off destination); re-archived on completion. -->
<!-- Previously archived: 2026-02-24 | Reason: All work items complete (CLN-001) -->

# Clone

| ID | Owner | Status |
|----|-------|--------|
| CLN | @joshuaboys | Complete |

## Purpose

Clone git repositories into organized directory structures, creating parent directories as needed and outputting the final path for shell integration. Support occasional one-off destinations without changing global `projectDir` / `structure` config.

## In Scope

- Shell out to `git clone` with the resolved URL and target path
- Create parent directories with safe permissions
- Detect and skip already-cloned repos
- Support shallow clones (`--depth=1`)
- Output the cloned path to stdout for the shell wrapper to `cd` into
- Update the project index after successful clone
- Optional one-off destination so a single clone can land outside `projectDir` + `structure` without mutating config

## Out of Scope

- Parallel cloning of multiple repos (future enhancement)
- Pull/fetch operations on existing repos
- Changing default `projectDir` / `structure` semantics for the common case
- Batch clone with mixed destinations

## Interfaces

**Depends on:**

- URL — to parse input and determine target path
- Index — to register cloned project
- Config — to read `projectDir`, `structure`, and `shallow` settings

**Exposes:**

- `clone` subcommand: `gx clone <repo> [dest]` — clone repo, return path

## Constraints

- Default path when no override is given remains `projectDir` + `structure` (unchanged)
- Override is per-invocation only — does not write config
- Override path is still indexed so `gx <name>` / `gx resolve` work afterward
- Existing already-cloned / symlink refusal behavior still applies at the chosen target

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined
- [x] Destination CLI surface decided — positional (D-013)
- [x] Index naming when dest basename differs from repo name specified — dest basename (D-014)

## Work Items

### CLN-001: Implement clone command with directory creation and index update

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Clone repositories to organized paths, creating directories as needed, skipping duplicates, and registering in the project index
- **Expected Outcome:** `gx clone user/repo` clones to the correct directory, skips already-cloned repos, and updates the index
- **Validation:** (historical) `bun test tests/commands/clone.test.ts` — now covered by the Rust port's tests
- **Files:** (historical) `src/commands/clone.ts` — current: `crates/gx/src/commands/clone.rs`

### CLN-002: Optional one-off clone destination

| Field | Value |
|-------|-------|
| Status | Complete: 2026-08-12 |
| Confidence | high |

- **Intent:** Let a single `gx clone` land at an explicit path without changing `projectDir` or `structure`
- **Expected Outcome:** `gx clone org/repo` still uses the configured layout; `gx clone org/repo <dest>` clones into `<dest>`, indexes that path under the destination's basename, and the shell `cd`s there; invalid destinations fail with a usage error and no config is written
- **Validation:** `cargo test --workspace` (`crates/gx/tests/clone_dest.rs` end-to-end, `resolve_target` unit tests, `clone_empty_dest` / `clone_too_many_args` snapshots)
- **Files:** `crates/gx/src/commands/clone.rs`, `crates/gx/src/cli.rs`, `crates/gx/src/path.rs`, `crates/gx/tests/clone_dest.rs`, `README.md`
- **Notes:** Positional destination with git-like semantics (relative paths resolve against the cwd). The shared `lexical_resolve` helper moved from `commands/index_repos.rs` to `path.rs` so clone and `gx index` resolve paths identically. Agent routing (`GX_AGENT`) shapes the default layout only; an explicit destination is taken literally.
