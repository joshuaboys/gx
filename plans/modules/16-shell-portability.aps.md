# Shell Portability

| Field | Value |
|-------|-------|
| Module ID | SHELL |
| Status | Draft |
| Version | v5 |
| Owner | @joshuaboys |
| Depends On | Shell Plugin, CLI |

## Purpose

Expand `gx` beyond oh-my-zsh to increase adoption.

## Outcomes

- Native shell integration for bash and fish
- Cross-shell completion support
- Consistent `gx jump` UX across shells

## Work Items

### SHELL-001: Bash integration
- Completion + jump function + plugin docs

### SHELL-002: Fish integration
- Completion + function wrappers

### SHELL-003: Compatibility matrix
- Test shell behavior in CI and document differences
