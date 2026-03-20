# Shell Portability

| ID    | Owner       | Status   |
| ----- | ----------- | -------- |
| SHELL | @joshuaboys | Complete |

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

All three shells (zsh, bash, fish) are implemented in `src/commands/shell-init.ts` with full parity:

- `eval "$(gx shell-init)"` auto-detects the parent shell
- `gx shell-init zsh|bash|fish` generates shell-specific integration
- Tab completions for all commands, project names, and config keys in each shell
- `plugin/gx.plugin.zsh` remains as an oh-my-zsh compatibility shim

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

- [x] SHELL-1: Implement `gx shell-init bash` with cd wrapper and completions
- [x] SHELL-2: Implement `gx shell-init fish` with cd wrapper and completions
- [x] SHELL-3: Auto-detect parent shell in `gx shell-init` (no argument)
