# Project Hooks

| ID | Owner | Status |
|----|-------|--------|
| HOOK | @joshuaboys | Future |

## Purpose

Allow projects to define commands that run automatically when a user navigates into them via `gx`. A `.gx.json` file in the project root specifies `onEnter` hooks (e.g., `fnm use`, `source .venv/bin/activate`, environment variable loading), enabling per-project environment setup without manual steps.

## In Scope

- `.gx.json` configuration file in project root with `onEnter` command array
- Execute `onEnter` commands in order when `gx <name>` enters a project
- Trust/allow mechanism: first encounter prompts user to trust project hooks (direnv-style)
- `gx trust <name>` — mark a project's hooks as trusted
- `gx untrust <name>` — revoke trust for a project's hooks
- Trust state persisted in `~/.config/gx/trusted.json`
- Integration with agent-vault for secret injection into hook environment
- Shell plugin modification to invoke hooks after `cd`

## Out of Scope

- `onExit` hooks (running commands when leaving a project)
- Hooks for non-navigation events (e.g., pre-commit, pre-push)
- GUI or interactive hook configuration editor
- Hook execution timeout or sandboxing (trust model assumes user responsibility)
- Hooks for `gx clone` (only `gx <name>` navigation triggers hooks)

## Interfaces

**Depends on:**

- Shell Plugin (05) — hooks execute in shell context after `cd`, requires plugin modification
- agent-vault (external) — optional secret management integration

**Exposes:**

- `.gx.json` file format specification
- `loadHooks(projectPath: string): HookConfig | null` — read hooks from project
- `isTrusted(name: string): boolean` — check trust status
- `trust` / `untrust` subcommands in CLI
- Hook execution in shell plugin flow

## Constraints

- Hooks must never run without explicit user trust — security is paramount
- `.gx.json` must be a simple, well-documented format (not a full scripting language)
- Hook execution failures must not prevent navigation — warn but still `cd`
- Trust is per-project, not per-hook — trusting a project trusts all its hooks
- agent-vault integration is optional — hooks work without it

## Ready Checklist

Change status to **Ready** when:

- [ ] `.gx.json` schema defined and documented
- [ ] Trust/allow security model designed
- [ ] agent-vault integration approach determined
- [ ] Shell plugin hook injection point identified
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- this module is in Future status._
