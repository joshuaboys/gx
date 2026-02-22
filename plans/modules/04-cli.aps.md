# CLI

| ID | Owner | Status |
|----|-------|--------|
| CLI | @joshuaboys | Complete |

## Purpose

Entry point for the `gx` binary. Routes subcommands, parses arguments, and orchestrates the URL, Clone, and Index modules.

## In Scope

- Subcommand routing: `clone`, `ls`, `rebuild`, `config`, `resolve`
- Argument parsing and validation
- `gx resolve <name>` — output path to stdout (used by zsh plugin)
- `gx clone <repo>` — clone and output path
- `gx ls` — list indexed projects
- `gx rebuild` — rescan and rebuild index
- `gx config [set <key> <val>]` — show or update config
- `gx --help` and `gx --version`
- Exit codes (0 success, 1 error)
- Error messages to stderr, paths to stdout

## Out of Scope

- Shell `cd` (handled by zsh plugin)
- Tab completion logic (handled by zsh plugin)

## Interfaces

**Depends on:**

- URL — for clone command
- Clone — for clone command
- Index — for resolve, ls, rebuild commands
- Config — for config command

**Exposes:**

- Binary entry point compiled via `bun build --compile`

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

### CLI-001: Implement subcommand routing and argument parsing

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Route CLI arguments to the correct command handlers with proper error handling and exit codes
- **Expected Outcome:** `gx clone`, `gx ls`, `gx rebuild`, `gx config`, and `gx resolve` all dispatch correctly, with help text and version output
- **Validation:** `bun test tests/types.test.ts`
- **Files:** `src/index.ts`, `src/commands/clone.ts`, `src/commands/ls.ts`, `src/commands/rebuild.ts`, `src/commands/config.ts`, `src/commands/resolve.ts`

### CLI-002: Compile to standalone binary

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Produce a single executable binary via `bun build --compile` that can be placed on PATH
- **Expected Outcome:** Running `bun build --compile` produces a working `gx` binary with all subcommands functional
- **Validation:** `bun build --compile src/index.ts --outfile gx && ./gx --help`
