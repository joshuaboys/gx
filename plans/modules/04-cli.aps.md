# CLI

| ID | Owner | Status |
|----|-------|--------|
| CLI | @joshuaboys | Draft |

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

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

*No tasks yet — module is Draft*

## Execution *(optional)*

Steps: [../execution/CLI.steps.md](../execution/CLI.steps.md)
