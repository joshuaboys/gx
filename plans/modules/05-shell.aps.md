# Shell Plugin

| ID | Owner | Status |
|----|-------|--------|
| SHL | @joshuaboys | Draft |

## Purpose

Zsh plugin that wraps the `gx` binary to provide shell-level integration — specifically `cd` (which a subprocess cannot do) and tab completion.

## In Scope

- `gx <name>` intercept: call `gx resolve`, then `cd` to result
- `gx clone <repo>` intercept: call `gx clone`, then `cd` to result
- Pass-through for all other subcommands (`ls`, `rebuild`, `config`)
- Zsh tab completion using index data (`gx resolve --list`)
- Installable as oh-my-zsh custom plugin
- Override oh-my-zsh git plugin's `gcd` alias if present

## Out of Scope

- Bash, Fish, or PowerShell support (future)
- Plugin manager support beyond oh-my-zsh (future)

## Interfaces

**Depends on:**

- CLI binary — calls `gx resolve`, `gx clone`, etc.

**Exposes:**

- `gx` shell function (wraps binary)
- `_gx` completion function

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

*No tasks yet — module is Draft*

## Execution *(optional)*

Steps: [../execution/SHL.steps.md](../execution/SHL.steps.md)
