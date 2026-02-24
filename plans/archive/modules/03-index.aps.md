<!-- Archived: 2026-02-24 | Reason: All work items complete -->

# Project Index

| ID | Owner | Status |
|----|-------|--------|
| IDX | @joshuaboys | Complete |

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

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

### IDX-001: Implement persistent project index with CRUD operations

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Provide a JSON-backed index that supports add, resolve, list, and rebuild operations for project tracking
- **Expected Outcome:** Index persists to `~/.config/gx/index.json`, supports name-to-path resolution, and rebuild scans the project directory
- **Validation:** `bun test tests/lib/index.test.ts`
- **Files:** `src/lib/index.ts`, `tests/lib/index.test.ts`

### IDX-002: Implement config management

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Load and persist user configuration with sensible defaults for projectDir, defaultHost, and structure
- **Expected Outcome:** Config loads from `~/.config/gx/config.json` with defaults, supports get/set operations
- **Validation:** `bun test tests/lib/config.test.ts`
- **Files:** `src/lib/config.ts`, `tests/lib/config.test.ts`
