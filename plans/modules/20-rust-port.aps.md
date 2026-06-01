# Rust Port

| ID  | Owner       | Status    |
| --- | ----------- | --------- |
| RST | @joshuaboys | Committed |

## Purpose

Re-implement gx as a Rust binary that is byte-for-byte compatible with the current TypeScript implementation: same CLI surface, same `~/.config/gx/` schemas, same `gx shell-init` output. This removes the 50–90 MB Bun-compile binary, drops Bun as a contributor and CI prerequisite, opens a path to Windows support, and lowers cold-start latency on every shell-completion call.

## In Scope

- Port every shipped command: `clone`, `ls`, `index`, `rebuild`, `config`, `open`, `init`, `shell-init`, `resolve`, `recent`, `resume`
- Port every library module: URL parsing, path mapping, fuzzy matching, project index (load/save/scan/rebuild), name resolution, project-type detection, templates, time formatting, errors
- Reproduce `index.json` and `config.json` schemas exactly (field names, types, defaults, JSON formatting)
- Reproduce `gx shell-init` output verbatim for zsh, bash, and fish
- Snapshot harness that captures the TS binary's output and asserts the Rust binary matches
- Cargo-based CI matrix replacing Bun on `ubuntu-latest` and `macos-latest`
- Cross-compiled prebuilt assets for `linux-x64`, `linux-aarch64`, `darwin-x64`, `darwin-aarch64`
- Update `install.sh` to fetch new asset names and drop the Bun build-from-source fallback
- Rewrite `AGENTS.md` for Rust conventions

## Out of Scope

- Any v3+ planned feature (`fork`, `sync`, `dash`, hooks, TUI, tutorial) — port first, build new features after
- Any behavior change (different defaults, new flags, different error messages or exit codes)
- Switching `gx shell-init` output to clap-generated completions
- Adding async or parallelism — every current command is sequential
- Windows support in `install.sh` (the Rust binary will compile for Windows, but installer work is deferred)
- Replacing git subprocess calls with libgit2

## Interfaces

**Depends on:**

- Every existing module — this is a wholesale re-implementation rather than a new feature

**Exposes (post-port):**

- A `gx` binary on `$PATH` with identical CLI behavior
- The same `~/.config/gx/{config,index}.json` schemas
- The same shell wrapper code emitted by `gx shell-init`

## Constraints

- **Behavior parity:** any deviation from the TS binary in stdout, stderr, exit code, file contents, or shell-init output is a regression and must be fixed before that command's port can land
- **Schema parity:** an `index.json` written by either binary must round-trip through the other without semantic change
- **Atomic saves preserved:** `index.json` writes must continue to use temp-file-then-rename inside the same directory
- **Path-traversal validation preserved:** `..`, `.`, empty-segment, backslash, and NUL guards in URL parsing must be ported character-for-character
- **Fuzzy thresholds preserved:** `similarityThreshold = 0.7` default and `AUTO_JUMP_THRESHOLD = 0.85` constant
- **Git via subprocess only:** no `git2`/libgit2 dependency
- **No async runtime:** stay synchronous; defer `tokio` until a feature actually needs concurrency (`gx dash`)
- **Reversible:** TS source must remain buildable through phase RST-6 so the experiment can be abandoned (RST-7 is the cutover that removes it)

## Ready Checklist

Change status to **Ready** when:

- [x] Decision recorded as ADR — see `decisions/010-rust-port.md`
- [ ] Crate selection finalised (clap, serde, serde_json, walkdir, tempfile, regex, thiserror, dirs)
- [ ] Snapshot-harness shape agreed (`assert_cmd` + `insta`, fixtures under `tests/fixtures/`)
- [ ] Workspace layout decided (single crate at root vs. `crates/gx/` with virtual manifest)
- [ ] Phase ordering validated against module dependencies
- [ ] Distribution asset naming agreed (preserve `gx-{linux,darwin}-{x64,aarch64}`)

## Work Items

- [x] **RST-1:** Cargo scaffolding and CI matrix
  - Cargo manifest, empty binary, `cargo {fmt,clippy,test,build}` jobs added to `ci.yml`
- [x] **RST-2:** Snapshot harness against the current TS binary
  - Harness in `crates/gx/tests/snapshots.rs` runs the binary under test in an
    isolated `HOME` populated from `crates/gx/tests/fixtures/`, captures
    stdout/stderr/exit, and asserts against goldens under
    `crates/gx/tests/snapshots/`. Tests are `#[ignore]`d so `cargo test` stays
    green during the port; goldens get verified by pointing `GX_SNAPSHOT_BIN`
    at the freshly built TS binary via `tools/run-ts-snapshots.sh`, which is
    also wired into CI as the `snapshots` job. Initial coverage:
    deterministic read-only surfaces (`--help`, `--version`,
    `shell-init zsh|bash|fish`, `shell-init <bad>`, `ls`, `ls` empty,
    `resolve --list`, `resolve <missing>`, `recent`, `recent -n`, `config`
    show with/without fixture, fallback resolve of unknown name). Mutating-
    command goldens are added as those commands get ported in RST-5/RST-6.
- [x] **RST-3:** Pure-logic ports
  - `crates/gx/src/{types,errors,time,url,fuzzy,detect,templates,config,path}.rs`
    port the eight TS pure-logic modules with parity unit tests (111 total
    across config/detect/fuzzy/path/templates/time/url, mirroring
    `tests/lib/`). `effective_project_dir_for` / `to_path_for` /
    `validate_agent` take the agent string explicitly so tests don't depend
    on global env state. Atomic config save preserves the temp-then-rename
    invariant; pretty-print emits 2-space indent + trailing newline matching
    TS. Deps added: `serde`, `serde_json`, `regex`, `thiserror`, `dirs`.
- [x] **RST-4:** `index_store` port
  - `crates/gx/src/index_store.rs` ports `ProjectIndex` with the full TS
    surface: `load`/`save`/`add`/`merge`/`resolve`/`touch`/`recent`/`list`/
    `names`/`rebuild`/`scoped_rebuild`/`additive_scan`/`get_remote_url`.
    `types::Index::projects` is now an `indexmap::IndexMap` so a canonical
    `index.json` round-trips byte-equal (locked in by
    `round_trip_is_byte_equal`). Atomic save reuses
    `config::atomic_write` (temp-file-then-rename in the same directory).
    Scanner mirrors the TS rules: max depth 10, skip
    `{node_modules,vendor,target,.build,dist,build}` and hidden dirs,
    detect `.git` as directory OR file (worktrees), realpath-based
    visited set breaks symlink cycles (`scan_breaks_symlink_cycles`).
    `last_visited` stamping reproduces `new Date().toISOString()` shape
    (`YYYY-MM-DDTHH:MM:SS.sssZ`). 25 unit tests mirror
    `tests/lib/index.test.ts` including `get_remote_url` against a real
    `git init` repo. Dep added: `indexmap` with `serde` feature.
- [ ] **RST-5:** Read-only commands
  - `ls`, `recent`, `resolve`, `config`, `shell-init` wired through clap; snapshot harness green
- [ ] **RST-6:** Mutating commands
  - `clone`, `init`, `index`, `rebuild`, `open`, `resume` ported; full snapshot harness green; manual smoke on real repos
- [ ] **RST-7:** Distribution cutover
  - `install.sh` and release workflow ship the Rust binary; TS source removed; `AGENTS.md` rewritten for Rust
