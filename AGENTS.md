# Repository Guidelines

## Project Overview
`gx` is a Bun-based TypeScript CLI for managing Git project directories.

## Project Structure
- `src/index.ts`: CLI entrypoint and command routing.
- `src/commands/*.ts`: user-facing commands (`clone`, `open`, `init`, `shell-init`, etc.).
- `src/lib/*.ts`: shared logic (config, indexing, path/url parsing, templates, fuzzy matching).
- `tests/commands/` and `tests/lib/`: test suites that mirror source structure.
- `plugin/gx.plugin.zsh`: oh-my-zsh compatibility shim.
- `install.sh`: curl installer; `.github/workflows/ci.yml`: CI checks.

## Tooling — Bun, Not Node
Always use Bun. Never use Node.js, npm, yarn, pnpm, npx, jest, vitest, webpack, esbuild, dotenv, express, or execa.

| Task | Command |
|------|---------|
| Run a file | `bun <file>` |
| Install deps | `bun install` |
| Run a script | `bun run <script>` |
| Run a package | `bunx <package>` |
| Run tests | `bun test` |
| Type check | `bun x tsc --noEmit` |
| Build binary | `bun run build` |

Prefer Bun-native APIs: `Bun.file` over `node:fs`, `Bun.$\`cmd\`` over child_process/execa. Bun auto-loads `.env` — don't use dotenv.

## Development Commands
- `bun run dev`: run CLI from source.
- `bun run build`: compile standalone binary to `./gx`.
- Pre-PR check: `bun x tsc --noEmit && bun test && bun run build`.

## Coding Style
- TypeScript (ES modules), `strict` type checking enabled.
- 2-space indentation, semicolons, double quotes, trailing commas for multiline.
- Command handlers in `src/commands/`; reusable logic in `src/lib/`.
- Lowercase filenames with concise names (e.g. `shell-init.ts`, `config.ts`).

## Testing
- Framework: `bun:test` (`import { test, expect } from "bun:test"`).
- Test files: `*.test.ts`, mirroring source paths.
- Use temp directories with `beforeEach`/`afterEach` cleanup for filesystem tests.
- Prefer deterministic tests; mock external dependencies where practical.

## Commits & Pull Requests
- Conventional Commits: `feat:`, `fix:`, `docs:`, `ci:`, `chore:` with optional scopes.
- Short, imperative subjects.
- PRs: concise summary, linked issues, validation steps, doc updates for CLI changes.
- CI must pass on both `ubuntu-latest` and `macos-latest`.
