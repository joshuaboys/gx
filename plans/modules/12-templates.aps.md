# Project Templates

| ID | Owner | Status |
|----|-------|--------|
| TPL | @joshuaboys | Future |

## Purpose

Enable users to create new projects from template repositories via `gx new`. Templates are git repos that get cloned, reinitialized (fresh `.git`), and automatically indexed, giving users a fast project scaffolding workflow integrated with the gx ecosystem.

## In Scope

- `gx new <name> --template <template>` — create project from template
- `gx template add <name> <repo-url>` — register a template
- `gx template list` — list registered templates
- `gx template rm <name>` — remove a template
- Template registry stored in `~/.config/gx/templates.json`
- Clone template repo, remove `.git`, reinitialize with `git init`
- Auto-index the new project after creation
- Place new project in `projectDir` following the same structure rules as `gx clone`

## Out of Scope

- Template variable substitution or placeholders (e.g., `{{project-name}}`)
- Interactive template wizards or prompts
- Publishing or sharing templates (local registry only)
- Template versioning or update detection
- Cookiecutter/Yeoman-style template engines

## Interfaces

**Depends on:**

- Clone (02) — reuses URL parsing and `git clone` infrastructure
- Index (03) — auto-indexes newly created projects

**Exposes:**

- `newFromTemplate(name: string, template: string): Promise<string>` — create project, return path
- `addTemplate(name: string, url: string): Promise<void>` — register template
- `listTemplates(): Template[]` — list registered templates
- `removeTemplate(name: string): Promise<void>` — unregister template
- `new` subcommand in CLI
- `template` subcommand group in CLI

## Constraints

- Template clone must be a full clone (not shallow) to ensure all files are present
- `.git` directory must be completely removed and reinitialized — no history leakage from template
- Template names must be unique in the registry
- `gx new` without `--template` should list available templates and prompt (or error)

## Ready Checklist

Change status to **Ready** when:

- [ ] Template registry format defined
- [ ] Project naming and placement rules clear
- [ ] Git reinitialization strategy confirmed
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- this module is in Future status._
