# D-010: Port gx from TypeScript/Bun to Rust

| Status   | Date       | Owner       |
| -------- | ---------- | ----------- |
| Proposed | 2026-05-03 | @joshuaboys |

## Context

gx today is a TypeScript CLI compiled to a single binary by `bun build --compile`. The architecture has served v1–v3, but several frictions are accumulating:

- **Binary size.** A `bun --compile` artifact embeds the Bun runtime — current artifacts run 50–90 MB for ~1.8K LOC of source. `index.aps.md` already flags this in its risk table.
- **Cold-start latency.** Shell completions call `gx resolve --list` on every Tab. Bun's startup is fast for a JS runtime, but a Rust binary is an order of magnitude faster, which matters at this call frequency.
- **Build-time dependencies.** Contributors and CI need Bun. `install.sh` falls back to "install Bun, then build" when no prebuilt asset exists. A statically linked Rust binary removes that fallback path.
- **Cross-compilation.** Bun does not produce Windows binaries. Rust does. `install.sh` does not currently target Windows, but the binary should not be the blocker.
- **Test infrastructure.** `bun:test` is fine but ties tests to Bun. `cargo test` is hermetic on every supported platform with no external setup.

## Decision

Port gx to Rust under strict behavior parity. The port:

- Replaces the Bun toolchain entirely; the shipped binary is `cargo build --release` output
- Preserves the CLI surface, `~/.config/gx/{config,index}.json` schemas, and `gx shell-init` output verbatim
- Keeps git operations as `git` subprocess calls (no `git2`/libgit2)
- Stays synchronous (no `tokio`) until a feature actually needs concurrency
- Lands incrementally on a long-lived `rust-port` branch with a snapshot harness asserting every command's output matches the TS binary's, so the work is reversible at any phase boundary

Work is tracked under module `RST` (`plans/modules/20-rust-port.aps.md`) in seven phases (RST-1 through RST-7).

## Crate Selection

| Concern             | Choice                              | Rationale                                                                                                  |
| ------------------- | ----------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| CLI parsing         | `clap` 4 (derive)                   | Mature, ergonomic, generates good `--help`                                                                 |
| JSON                | `serde` + `serde_json`              | Pretty-print matches TS `JSON.stringify(.., null, 2)` output                                               |
| Atomic file writes  | `tempfile::NamedTempFile`           | Same-dir tempfile + atomic rename; preserves the current invariant                                         |
| Filesystem walk     | `walkdir`                           | Configurable depth and skip rules; canonicalise for symlink-cycle detection                                |
| Regex               | `regex`                             | URL, agent, and segment validation                                                                         |
| Path utilities      | `std::path` + `dirs`                | `dirs::home_dir()` replaces `os.homedir()`                                                                 |
| Errors              | `thiserror`                         | Typed errors with `exit_code()`; `anyhow` only at `main`                                                   |
| Fuzzy matching      | hand-port of the current Jaro-Winkler | TS impl is case-insensitive with prefix cap of 4 and `p=0.1`; verify against `strsim` before substituting |
| Tests               | `assert_cmd` + `insta` + `tempfile` | End-to-end snapshots match the parity-first plan                                                           |

Explicitly rejected:

- **`git2`/libgit2** — adds a C dependency for one-shot subprocess workloads
- **`tokio`** — premature; no current command benefits
- **`clap_complete` for `gx shell-init`** — would change the verbatim output users have already added to their shells

## Behavior Parity Contracts

These are the invariants the snapshot harness enforces:

1. `~/.config/gx/index.json` and `~/.config/gx/config.json` schemas — field names, types, defaults, JSON formatting (2-space indent, trailing newline)
2. `gx resolve <name>` writes the path to stdout and hints to stderr; shell wrappers depend on this split
3. `gx clone` writes the final path to stdout; shell wrappers `cd` into it
4. `gx shell-init {zsh,bash,fish}` output is byte-identical
5. Atomic index save (temp file in the same directory, then rename)
6. Path-traversal validation: `..`, `.`, empty segments, backslash, NUL all rejected
7. Fuzzy thresholds: `0.7` similarity default, `0.85` auto-jump
8. Exit codes: `CommandError(message, exitCode=1)` semantics map to typed errors with `exit_code()` defaulting to 1
9. `GX_AGENT` validation regex unchanged

## Consequences

**Positive**

- Smaller binaries, faster cold starts, simpler install flow
- Removes Bun as a contributor and CI prerequisite
- Opens a path to Windows support
- Standard Rust tooling (`cargo`, `rustfmt`, `clippy`) replaces a Bun-specific toolchain

**Negative**

- Source size grows roughly 1.5–2× due to explicit error types and trait boilerplate
- Loses Bun-native conveniences (`Bun.file`, `Bun.spawn`, `Bun.write`)
- Cultural shift: `AGENTS.md` and project conventions need rewriting
- Contributor pool now needs Rust familiarity rather than TypeScript

**Neutral**

- TS implementation stays buildable through phase 5; the experiment is reversible
- Shell-plugin users see no change while `shell-init` output is preserved verbatim

## Open Questions

- **Q-1:** Workspace layout — single crate at root vs. `crates/gx/` with virtual manifest. Recommendation: start single-crate, restructure when a second crate exists.
- **Q-2:** Bin name during transition — ship as `gx-rs` for one release before promoting, or feature-flag a single binary? Recommendation: `gx-rs` shadow release, then promote at phase 7.
- **Q-3:** MSRV — pin to the current stable at port time (likely 1.75+), no nightly features.
- **Q-4:** `gx shell-init` — verbatim port (chosen) vs. switch to `clap_complete`. Verbatim wins because user shells already eval the existing strings.
- **Q-5:** Windows — produce binaries in CI from phase 7, but defer `install.sh` Windows support to a separate effort.

## Supersedes

This decision modifies the constraint in `index.aps.md` ("TypeScript + Bun only (no Go, no Node)") to "Rust only (no JS runtime)" once the port reaches phase 6 (TS source removed).
