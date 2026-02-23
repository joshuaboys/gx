# Shell Plugin

| ID | Owner | Status |
|----|-------|--------|
| SHL | @joshuaboys | Complete |

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

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

### SHL-001: Implement zsh plugin with cd wrapper and tab completion

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Provide a zsh shell function that intercepts `gx` calls to enable `cd` on clone and resolve, plus tab completion for project names
- **Expected Outcome:** `gx <name>` changes directory, `gx clone <repo>` clones and changes directory, tab completion lists indexed projects
- **Validation:** `source plugin/gx.plugin.zsh && type gx | head -1` outputs "gx is a shell function"
- **Files:** `plugin/gx.plugin.zsh`

### SHL-002: Fix resolveGxBin() producing invalid path in dev mode

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** `gx shell-init` run via `bun src/index.ts` baked `_GX_BIN` as `$cwd/bun` instead of the compiled binary path, breaking all shell commands
- **Expected Outcome:** `resolveGxBin()` detects dev mode (argv[0] is `bun`/`node`) and falls back to `Bun.which("gx")` to find the compiled binary on PATH
- **Validation:** `bun run src/index.ts shell-init | grep _GX_BIN` outputs the compiled binary path, not a cwd-relative bun path
- **Files:** `src/commands/shell-init.ts`
