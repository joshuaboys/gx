# Repository Guidelines

## Project Overview

`gx` is a Rust CLI for managing Git project directories. It compiles to a
single statically-linkable binary with no runtime dependencies.

## Project Structure

- `crates/gx/src/main.rs`: binary entrypoint (`gx::cli::run`).
- `crates/gx/src/cli.rs`: argument dispatch, `--help`/`--version`, command routing.
- `crates/gx/src/commands/*.rs`: user-facing commands (`clone`, `open`, `init`, `shell_init`, etc.).
- `crates/gx/src/*.rs`: shared logic — `config`, `index_store`, `path`, `url`, `templates`, `fuzzy`, `resolve_name`, `detect`, `time`, `types`, `errors`.
- `crates/gx/tests/`: integration tests — `smoke.rs` and the behaviour-parity `snapshots.rs` harness (goldens under `tests/snapshots/`, fixtures under `tests/fixtures/`). Unit tests live in `#[cfg(test)]` modules next to the code.
- `plugin/gx.plugin.zsh`: oh-my-zsh compatibility shim.
- `install.sh`: curl installer (downloads + checksum-verifies prebuilt release assets).
- `.github/workflows/ci.yml`: fmt/clippy/test/build. `.github/workflows/release.yml`: tag-triggered cross-platform binaries.

## Tooling — Cargo

| Task            | Command                                            |
| --------------- | -------------------------------------------------- |
| Run from source | `cargo run -p gx -- <args>`                        |
| Build (debug)   | `cargo build`                                      |
| Build (release) | `cargo build --release`                            |
| Run tests       | `cargo test --workspace`                           |
| Format          | `cargo fmt`                                        |
| Lint            | `cargo clippy --all-targets --workspace -- -D warnings` |

Prefer the standard library and the existing dependency set (`serde`,
`serde_json`, `regex`, `thiserror`, `dirs`, `indexmap`). Git operations run as
`std::process::Command` subprocesses — do not add `git2`/libgit2. Stay
synchronous; do not add an async runtime until a feature genuinely needs one.

## Development Commands

- `cargo run -p gx -- <args>`: run the CLI from source.
- `cargo build --release`: compile the standalone binary to `target/release/gx`.
- Pre-PR check: `cargo fmt --check && cargo clippy --all-targets --workspace -- -D warnings && cargo test --workspace`.

## Coding Style

- Rust 2021, MSRV 1.75. `rustfmt` defaults (4-space indent); keep `cargo fmt` clean.
- No `clippy` warnings (`-D warnings` in CI).
- Command handlers in `crates/gx/src/commands/`; reusable logic in sibling modules.
- Snake_case module files mirroring the command name (e.g. `shell_init.rs`, `config_cmd.rs`).
- Typed errors via `GxError`/`GxResult`; reserve `?`-propagation for failures and print user-facing messages through the error model.

## Testing

- Unit tests: `#[cfg(test)] mod tests` next to the code under test.
- Integration/parity: `crates/gx/tests/snapshots.rs` runs the binary in an
  isolated `HOME` from `tests/fixtures/` and asserts stdout/stderr/exit
  byte-for-byte against goldens in `tests/snapshots/`. The goldens are the
  behaviour contract — update them deliberately (e.g. `INSTA_UPDATE=always`)
  and review every diff.
- Use `tempfile::TempDir` for filesystem tests; keep tests deterministic
  (the harness scrubs binary paths and relative times).

## Commits & Pull Requests

- Conventional Commits: `feat:`, `fix:`, `docs:`, `ci:`, `chore:` with optional scopes.
- Short, imperative subjects.
- PRs: concise summary, linked issues, validation steps, doc updates for CLI changes.
- CI must pass on both `ubuntu-latest` and `macos-latest`.

## Notifications

This project is monitored by superterm. When you need user input, are waiting
for confirmation, or have completed a long-running task, notify the dashboard:

```sh
superterm notify
superterm notify --title "Need help"
superterm notify --title "Done" --body "All tests passed"
```

If superterm is not installed, fall back to a terminal bell:

```sh
printf '\a'
```
