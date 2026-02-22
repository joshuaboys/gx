# Project Index

| ID | Owner | Status |
|----|-------|--------|
| IDX | @joshuaboys | Draft |

## Purpose

Maintain a persistent index of all known projects, enabling fast name-to-path resolution for jumping between projects and tab completion.

## In Scope

- Read/write index file (`~/.config/gx/index.json`)
- Add project entries (name, path, url, timestamp)
- Resolve project name to filesystem path
- List all indexed projects
- Rebuild index by scanning `projectDir` for git repos
- Handle name collisions (warn, last-write-wins)

## Out of Scope

- Fuzzy matching or search ranking (future enhancement)
- Project metadata beyond name/path/url/timestamp

## Interfaces

**Depends on:**

- Config — to read `projectDir` for rebuild scanning

**Exposes:**

- `load(): Promise<Index>` — load index from disk
- `save(index: Index): Promise<void>` — persist index
- `add(entry: IndexEntry): Promise<void>` — add or update project
- `resolve(name: string): string | null` — name to path
- `list(): IndexEntry[]` — all projects
- `rebuild(projectDir: string): Promise<Index>` — scan and rebuild

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

*No tasks yet — module is Draft*

## Execution *(optional)*

Steps: [../execution/IDX.steps.md](../execution/IDX.steps.md)
