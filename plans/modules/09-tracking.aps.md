# Project Tracking

| ID | Owner | Status |
|----|-------|--------|
| TRK | @joshuaboys | Draft |

## Purpose

Track when projects are visited so users can quickly return to recently used projects. Every `gx <name>` or `gx clone` records a `lastVisited` timestamp, enabling recency-sorted listing and a context-rich resume workflow.

## In Scope

- Add `lastVisited` timestamp field to index entries
- Update `lastVisited` on every `gx <name>` jump and `gx clone`
- `gx recent` — list projects sorted by most recently visited
- `gx recent -n <N>` — limit output to last N projects
- `gx resume <name>` — jump to project and print context (current branch, dirty file count, last commit summary)
- Internal `gx touch <name>` subcommand for shell plugin to record visits without resolving
- Tab completion for `gx recent` and `gx resume`

## Out of Scope

- Visit frequency tracking or scoring (recency only, not frecency)
- Automatic pruning of stale entries from the index
- Tracking time spent in a project (duration, not just timestamp)
- Shell prompt integration showing current project name

## Interfaces

**Depends on:**

- Index (03) — reads and writes index entries, extends `IndexEntry` with `lastVisited` field

**Exposes:**

- `touch(name: string): Promise<void>` — update `lastVisited` for a project
- `recent(limit?: number): IndexEntry[]` — return projects sorted by `lastVisited` descending
- `resume(name: string): ResumeContext` — resolve project path and gather git context
- `recent` subcommand in CLI
- `resume` subcommand in CLI
- `touch` subcommand in CLI (internal, used by shell plugin)

## Constraints

- Must not break existing index.json format — `lastVisited` is optional (null for never-visited projects)
- `gx resume` must not fail if the project directory has been deleted — report "project missing" gracefully
- `gx recent` with no visits yet should fall back to `clonedAt` ordering
- `gx touch` should be fast (no stdout, minimal I/O) since it runs on every shell navigation

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Index entry schema extension designed (backward-compatible)
- [ ] Shell plugin integration for `touch` defined
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
