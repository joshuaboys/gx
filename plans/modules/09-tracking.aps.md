# Project Tracking

| ID  | Owner       | Status   |
| --- | ----------- | -------- |
| TRK | @joshuaboys | Complete |

## Purpose

Track when projects are visited so users can quickly return to recently used projects. Every `gx <name>` or `gx clone` records a `lastVisited` timestamp, enabling recency-sorted listing and a context-rich resume workflow.

## In Scope

- Add `lastVisited` timestamp field to index entries
- Update `lastVisited` on every `gx <name>` jump and `gx clone`
- `gx recent` — list projects sorted by most recently visited
- `gx recent -n <N>` — limit output to last N projects
- `gx resume <name>` — jump to project and print context (current branch, dirty file count, last commit summary)
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

- `touch(name: string): boolean` — update `lastVisited` for a project (internal method on ProjectIndex)
- `recent(limit?: number): Array<[string, IndexEntry]>` — return projects sorted by `lastVisited` descending
- `getResumeContext(dir: string): ResumeContext | null` — gather git context for a project directory
- `recent` subcommand in CLI
- `resume` subcommand in CLI

## Design Decision

The original spec included a separate `gx touch` CLI command for the shell plugin to call after each `cd`. During design review, this was eliminated: `resolve` now updates `lastVisited` as a side effect, removing an entire subprocess spawn per navigation.

## Constraints

- Must not break existing index.json format — `lastVisited` is optional (absent for never-visited projects)
- `gx resume` must not fail if the project directory has been deleted — report "project missing" gracefully
- `gx recent` with no visits yet should fall back to `clonedAt` ordering

## Work Items

- [x] TRK-1: Add `lastVisited` field and `touch()` method to ProjectIndex
- [x] TRK-2: Update `resolve` to call `touch()` on match
- [x] TRK-3: Implement `gx recent` command
- [x] TRK-4: Implement `gx resume` command with git context
- [x] TRK-5: Add shell wrapper integration for `resume`
- [x] TRK-6: Add tab completion for `recent` and `resume`

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Index entry schema extension designed (backward-compatible)
- [x] Shell plugin integration defined (resolve updates lastVisited as side effect)
- [x] Dependencies identified
- [x] At least one task defined
