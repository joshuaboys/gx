# Repository Guidelines

## Project Overview

`gx` is a Git project manager CLI. The shipped binary is the Rust port under
`crates/gx`; the Bun/TypeScript implementation remains in-tree as the parity
source while the port is completed.

## Project Structure

- `src/index.ts`: CLI entrypoint and command routing.
- `src/commands/*.ts`: user-facing commands (`clone`, `open`, `init`, `shell-init`, etc.).
- `src/lib/*.ts`: shared logic (config, indexing, path/url parsing, templates, fuzzy matching).
- `tests/commands/` and `tests/lib/`: test suites that mirror source structure.
- `crates/gx/src/`: Rust CLI, commands, and shared modules.
- `crates/gx/tests/snapshots/`: Rust parity snapshots captured from the TypeScript binary.
- `plugin/gx.plugin.zsh`: oh-my-zsh compatibility shim.
- `install.sh`: curl installer; `.github/workflows/ci.yml`: CI checks.

## Tooling — Bun and Cargo, Not Node

Use Bun for TypeScript work and Cargo for Rust work. Never use Node.js, npm, yarn, pnpm, npx, jest, vitest, webpack, esbuild, dotenv, express, or execa.

| Task                         | Command                                                 |
| ---------------------------- | ------------------------------------------------------- |
| Run a TypeScript file        | `bun <file>`                                            |
| Install TypeScript deps      | `bun install`                                           |
| Run a TypeScript script      | `bun run <script>`                                      |
| Run a TypeScript package     | `bunx <package>`                                        |
| Run TypeScript tests         | `bun test`                                              |
| Type check TypeScript        | `bun x tsc --noEmit`                                    |
| Build TypeScript parity bin  | `bun run build`                                         |
| Format Rust                  | `cargo fmt`                                             |
| Lint Rust                    | `cargo clippy --workspace --all-targets -- -D warnings` |
| Run Rust tests               | `cargo test --workspace`                                |
| Build shipped release binary | `cargo build --release --workspace`                     |

Prefer Bun-native APIs: `Bun.file` over `node:fs`, `Bun.$\`cmd\``over child_process/execa. Bun auto-loads`.env` — don't use dotenv.

## Development Commands

- `bun run dev`: run CLI from source.
- `bun run build`: compile TypeScript parity binary to `./gx`.
- `cargo build --release --workspace`: compile shipped Rust binary to `target/release/gx`.
- Pre-PR check: `bun x tsc --noEmit && bun test && bun run build && cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`.

## Coding Style

- TypeScript (ES modules), `strict` type checking enabled.
- Rust 2021, `rust-version = "1.75"`.
- 2-space indentation, semicolons, double quotes, trailing commas for multiline.
- Command handlers in `src/commands/`; reusable logic in `src/lib/`.
- Rust command handlers in `crates/gx/src/commands/`; reusable Rust logic in `crates/gx/src/*.rs`.
- Lowercase filenames with concise names (e.g. `shell-init.ts`, `config.ts`).

## Testing

- Framework: `bun:test` (`import { test, expect } from "bun:test"`).
- Test files: `*.test.ts`, mirroring source paths.
- Rust tests use `cargo test`; parity snapshots live in `crates/gx/tests/snapshots.rs`.
- Use temp directories with `beforeEach`/`afterEach` cleanup for filesystem tests.
- Prefer deterministic tests; mock external dependencies where practical.

## Commits & Pull Requests

- Conventional Commits: `feat:`, `fix:`, `docs:`, `ci:`, `chore:` with optional scopes.
- Short, imperative subjects.
- PRs: concise summary, linked issues, validation steps, doc updates for CLI changes.
- CI must pass on Linux and macOS for both TypeScript parity and Rust.

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
