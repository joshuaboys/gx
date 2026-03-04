# Tracking Feature Design

**Date:** 2026-03-04
**Module:** 09-tracking
**Branch:** feat/tracking

## Problem

gx has no memory of which projects you use. After cloning 50+ repos, there's no way to see what you worked on recently or quickly resume where you left off with context.

## Approach

Add tracking directly to the index (Approach A). `lastVisited` is an index concern — no separate tracking module. `resolve` updates the timestamp as a side effect, eliminating the need for a separate `touch` command.

## Data Model

`IndexEntry` gains one optional field:

```typescript
export interface IndexEntry {
  path: string;
  url: string;
  clonedAt: string;
  lastVisited?: string; // ISO 8601, absent = never visited
}
```

Backward compatible — existing index.json files load without migration.

## Index Changes

Two new methods on `ProjectIndex`:

- **`touch(name: string): boolean`** — sets `lastVisited` to now. Returns false if entry not found. (Internal method, no CLI command.)
- **`recent(limit?: number): Array<[string, IndexEntry]>`** — entries sorted by `lastVisited` descending, falling back to `clonedAt` for unvisited entries.

## Command Changes

### resolve (modified)

`resolve` now calls `touch(name)` and saves the index after resolving. This records visits without a separate subprocess.

### gx recent [-n N] (new)

Lists projects sorted by most recently visited. Defaults to all; `-n N` limits output. Displays relative timestamps ("2 hours ago", "3 days ago").

### gx resume \<name\> (new)

Resolves project path, prints git context, outputs path for shell cd. Calls `touch` internally.

Output format:

```
myproject (feat/auth) — 3 dirty files
  Last commit: a1b2c3d fix: handle empty token (2 hours ago)
```

Git queries: `git branch --show-current`, `git status --porcelain`, `git log -1 --format='%h %s (%cr)'`.

### clone (modified)

Sets `lastVisited` to same value as `clonedAt` on creation.

## Shell Integration

No shell changes needed for tracking — `resolve` handles it. Shell wrapper updated for `resume`:

```bash
resume)
    local output
    output=$("$_GX_BIN" resume "${@:2}")
    local target
    target=$(echo "$output" | tail -1)
    echo "$output" | head -n -1
    if [ -n "$target" ] && [ -d "$target" ]; then
        cd "$target"
    fi
    ;;
```

Tab completion arrays get `recent` and `resume` added.

## Error Handling

- `gx recent` with no visits: falls back to `clonedAt` ordering
- `gx resume` with deleted directory: `"project 'x' directory not found: /path"`, exit 1
- `gx resume` with unknown name: `"project 'x' not found in index"`, exit 1

## Out of Scope

- Frecency scoring (recency only)
- Automatic stale entry pruning
- Duration tracking
- Shell prompt integration
