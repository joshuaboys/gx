# Agent Scaffolding

| ID | Owner | Status |
|----|-------|--------|
| AGT | @joshuaboys | Complete |

## Purpose

Provide `gx init` to scaffold AI agent configuration (`.claude/` directory with `CLAUDE.md`) in the current project. Auto-detects project type and generates appropriate context for AI-assisted development.

## In Scope

- `gx init` — scaffold `.claude/CLAUDE.md` in the current working directory
- Auto-detect project type by presence of manifest files (package.json, Cargo.toml, go.mod, pyproject.toml, etc.)
- Templates for common project types: TypeScript/Bun, Rust, Go, Python
- Generic fallback template when project type is unrecognized
- `gx init --type <type>` to override auto-detection
- Scaffold `.claude/commands/` with common slash commands (plan, review)
- Refuse to overwrite existing `.claude/CLAUDE.md` without `--force` flag

## Out of Scope

- Support for AI agents other than Claude (Copilot, Cursor rules, etc. — future)
- Project-specific content generation (reading source files to infer patterns)
- Installing or configuring Claude Code itself
- Template customization or user-defined templates (future)

## Interfaces

**Depends on:**

- CLI — new `init` subcommand registration

**Exposes:**

- `detectProjectType(dir: string): ProjectType` — infer project type from manifest files
- `scaffold(dir: string, type: ProjectType, opts: ScaffoldOpts): Promise<void>` — write `.claude/` directory structure
- `init` subcommand in CLI

## Constraints

- Must work from any directory, not just indexed projects
- Must not overwrite existing files without explicit `--force`
- Templates must be embedded in the binary (no external template directory)
- Output should be minimal and useful — avoid bloated boilerplate
- Detection order: explicit `--type` flag > manifest file presence > generic fallback

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Project type detection rules defined
- [ ] Template content for at least TypeScript/Bun drafted
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
