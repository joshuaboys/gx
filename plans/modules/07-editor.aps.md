# Editor Integration

| ID | Owner | Status |
|----|-------|--------|
| EDT | @joshuaboys | Complete |

## Purpose

Add `gx open <name>` to open a project directory in the user's preferred editor. Respects standard environment variables and supports common editors with sensible defaults.

## In Scope

- `gx open <name>` — resolve project by name and open in editor
- `gx open` with no argument — open current directory's project (if indexed)
- `--editor <name>` flag to override the default editor for a single invocation
- Respect `$EDITOR` and `$VISUAL` environment variables
- Known-editor table mapping names to launch commands (code, cursor, vim, nvim, emacs, zed, etc.)
- Configurable default editor via `gx config set editor <name>`

## Out of Scope

- Opening specific files within a project (open the project root only)
- IDE project file generation (.idea, .vscode/settings.json, etc.)
- Remote/SSH editor sessions
- Editor plugin or extension management

## Interfaces

**Depends on:**

- Index — `resolve(name)` to get project path
- Config — to read default editor setting
- CLI — new `open` subcommand registration

**Exposes:**

- `openInEditor(projectPath: string, editor?: string): Promise<void>` — launch editor for a project path
- `resolveEditor(config: Config, override?: string): string` — determine which editor to use
- `open` subcommand in CLI

## Constraints

- Must not block the CLI process waiting for the editor to close (spawn detached for GUI editors)
- Terminal editors (vim, nvim, emacs -nw) must inherit stdin/stdout
- Must handle missing/invalid editor gracefully with actionable error message
- Editor resolution order: `--editor` flag > `gx config editor` > `$VISUAL` > `$EDITOR` > sensible default

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Known-editor table drafted
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
