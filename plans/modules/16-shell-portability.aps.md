# Shell Portability

| ID    | Owner       | Status      |
| ----- | ----------- | ----------- |
| SHELL | @joshuaboys | In Progress |

- Version: v5 (shipped early — landed alongside v3 work)
- Depends on: Shell Plugin, CLI

## Purpose

Expand `gx` beyond oh-my-zsh to increase adoption.

## In Scope

- Native shell integration for bash and fish
- Cross-shell completion support
- Consistent `gx <name>` UX for jumping across shells

## Out of Scope

- Changes to core `gx` behavior that are not related to shell portability or integration

## Interfaces

**Depends on:**

- Shell Plugin — existing zsh plugin as reference
- CLI — command surface

**Exposes:**

- Shell plugins for bash and fish via `gx shell-init <shell>`

## Implementation Notes

Shell integration for zsh, bash, fish and PowerShell is implemented in
`crates/gx/src/commands/shell_init.rs`:

- `eval "$(gx shell-init)"` detects the shell from `GX_SHELL_OVERRIDE` or `$SHELL` (login shell; does not yet query the parent process directly)
- `gx shell-init zsh|bash|fish|powershell` generates shell-specific integration
- Tab completions for all commands, project names, and config keys in each shell
- `plugin/gx.plugin.zsh` remains as an oh-my-zsh compatibility shim

### Usability regression coverage

The 2026-10-22 usability maintenance change covers clone failure propagation,
zsh resume, help forwarding and first-start installer navigation. Executable
Unix regression tests live in `crates/gx/tests/shell_behaviour.rs`; the four
shell output snapshots remain under `crates/gx/tests/snapshots/`. CI provisions
Bash, zsh and fish for execution tests on Linux and macOS. This maintenance does
not complete SHELL-3 or change the module status.

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

- [x] SHELL-1: Implement `gx shell-init bash` with cd wrapper and completions
- [x] SHELL-2: Implement `gx shell-init fish` with cd wrapper and completions
- [ ] SHELL-3: Auto-detect parent shell in `gx shell-init` (no argument) — currently uses `$SHELL` (login shell), not parent-process detection
