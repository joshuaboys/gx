# Rust Port

| ID  | Owner       | Status      |
| --- | ----------- | ----------- |
| RST | @joshuaboys | In Progress |

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
- [x] Crate selection finalised — `serde`, `serde_json`, `regex`, `thiserror`, `dirs`, `indexmap` locked in via RST-3/RST-4; `clap` lands with RST-5
- [x] Snapshot-harness shape agreed — `crates/gx/tests/snapshots.rs` with fixtures under `crates/gx/tests/fixtures/` and goldens under `crates/gx/tests/snapshots/` (RST-2)
- [x] Workspace layout decided — `crates/gx/` layout (RST-1 scaffolding)
- [x] Phase ordering validated against module dependencies — RST-1→7 sequence; pure-logic before index_store before commands
- [x] Distribution asset naming agreed — preserve `gx-{linux,darwin}-{x64,aarch64}` (RST-7 scope)

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
    Name resolution (`resolve-name.ts`) is the one library module deferred
    out of this batch — it composes `index_store` + `fuzzy`, so it lands on
    top of `index_store` in RST-4 rather than here.
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
    `crates/gx/src/resolve_name.rs` then ports `resolve-name.ts` on top:
    `resolve_project_name` returns the `Exact`/`Auto`/`Ambiguous`/`Missing`
    `ResolveResult` enum and `format_ambiguous`/`format_auto_match` reproduce
    the suggestion and auto-jump strings verbatim (`(score*100).toFixed(0)`
    percentages). 7 unit tests mirror `tests/lib/resolve-name.test.ts`;
    library unit-test total is now 143. The command-layer goldens for the
    `Ambiguous`/`Auto` stderr output land with the `resolve`/`open`/`resume`
    ports in RST-5/RST-6.
- [x] **RST-5:** Read-only commands
  - `crates/gx/src/cli.rs` dispatches argv (hand-rolled, not clap — see note
    below) and `crates/gx/src/commands/{ls,recent,resolve,config_cmd,shell_init}.rs`
    port the read-only surface plus `--help`/`--version` and the
    unknown-command-resolves-as-name fallback. Crate version bumped to `0.2.0`
    so the version/help banners match. All 15 RST-2 snapshot goldens are now
    live (the `#[ignore]`s are removed) and green against the Rust binary;
    `tools/run-ts-snapshots.sh` switched to `--include-ignored` so the TS
    binary stays verified against the same goldens. `cargo test --workspace`
    is the parity gate (143 unit + 1 smoke + 15 snapshots).
  - **Deviation from plan/ADR (clap):** the read-only commands are dispatched
    by a hand-rolled parser, not `clap`. The parity contract requires the
    verbatim hand-written `--help` text, custom `Usage:`/error strings, and a
    default arm that resolves an unknown token as a project name — all of
    which clap's generated help, version, and strict-subcommand error
    formatting would break. clap is reconsidered if/when a future feature
    needs richer parsing; for the port, byte-parity wins.
  - **Deviation (`_GX_BIN` resolution):** `shell-init` embeds
    `std::env::current_exe()` instead of a PATH lookup of `gx`. The TS port
    PATH-looks-up because Bun compiled binaries report an unreliable
    `argv[0]`; Rust's `current_exe()` is reliable, always resolves, and never
    emits the "not on PATH" warning. The value is scrubbed to `<BIN>` in
    snapshots, so output stays parity-clean.
- [x] **RST-6:** Mutating commands
  - `crates/gx/src/commands/{clone,init,index_repos,rebuild,open,resume}.rs`
    port the six mutating commands and the dispatcher wires their flag parsing
    (`--editor`, `--type`, `--force`, `-n`). git runs as a subprocess
    (`std::process::Command`, no `git2`); `clone`/`index` stamp
    `clonedAt`/`lastVisited` via the now-public `index_store::iso_now`;
    `index` lexically resolves paths (`path.resolve` parity, symlinks
    preserved); `open` ports the editor table, `$VISUAL`/`$EDITOR` precedence,
    and a `Bun.which`-equivalent PATH probe. Five deterministic error-path
    snapshot goldens added (`clone_no_arg`, `resume_no_arg`, `resume_missing`,
    `open_missing`, `init_bad_type`) — 20 snapshots now green against the Rust
    binary. 6 unit tests cover `resolve_editor`/`editor_info`. Manual smoke
    ran every mutating command (init, index, rebuild, clone success +
    already-exists, resume) through both binaries in isolated `HOME`s against
    real git repos: stdout, stderr, and `index.json` (modulo timestamps) are
    byte-identical. `cargo test --workspace`: 149 unit + 1 smoke + 20
    snapshots.
- [x] **RST-7:** Distribution cutover
  - `.github/workflows/release.yml` cross-compiles the Rust binary to all four
    targets (`gx-{linux,darwin}-{x64,aarch64}`) on native runners (linux-aarch64
    via the `gcc-aarch64-linux-gnu` cross-linker, darwin-x64 via the macOS SDK),
    publishes a `SHA256SUMS` manifest, and emits build-provenance attestations.
    `install.sh` downloads the prebuilt asset, verifies its SHA-256 against the
    manifest, and installs it — the Bun build-from-source fallback is gone (it
    errors with build-from-source-with-cargo guidance when no asset matches).
    `ci.yml` is Rust-only (fmt + clippy `-D warnings` + `cargo test` + release
    build on ubuntu/macos); the Bun `check` job and the TS-parity `snapshots`
    job are removed — `cargo test` runs the parity snapshots against the Rust
    binary directly. TS source is deleted (`src/`, `tests/`, `package.json`,
    `bun.lock`, `tsconfig.json`, `.husky/`, `tools/run-ts-snapshots.sh`); the
    snapshot goldens are frozen as the behaviour contract. `AGENTS.md` rewritten
    for cargo conventions; `README.md` install/build sections retargeted to
    Rust. The snapshot harness no longer references Bun.
