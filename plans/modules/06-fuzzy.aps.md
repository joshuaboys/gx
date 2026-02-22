# Fuzzy Matching

| ID | Owner | Status |
|----|-------|--------|
| FUZ | @joshuaboys | Complete |

## Purpose

When `gx <name>` fails to find an exact match in the index, fall back to fuzzy matching against all indexed project names. Rank candidates by similarity and either auto-jump (single strong match) or present an interactive selection (multiple matches).

## In Scope

- Fuzzy string matching against indexed project names
- Similarity scoring and candidate ranking
- Auto-jump when a single candidate exceeds a high confidence threshold
- Interactive selection when multiple candidates match (numbered list fallback)
- Optional fzf integration when fzf is available on PATH
- Configurable similarity threshold via `gx config`

## Out of Scope

- Matching against project URLs or paths (name only)
- Full-text search across project metadata
- Learning from user selection history (future enhancement)
- Fuzzy matching for subcommands or flags

## Interfaces

**Depends on:**

- Index — `list()` to get all project names for matching
- Config — to read similarity threshold setting

**Exposes:**

- `fuzzyMatch(query: string, names: string[]): FuzzyResult[]` — ranked list of matches with scores
- `selectMatch(results: FuzzyResult[]): Promise<string | null>` — interactive selection or auto-pick
- Integration into resolve flow — CLI falls through to fuzzy when exact match returns null

## Constraints

- Must not add external dependencies for fuzzy matching (implement in-house or use built-in string methods)
- Interactive selection must work in non-TTY environments (return null, no hang)
- fzf integration is optional — absence of fzf must not cause errors
- Auto-jump threshold must be conservative to avoid jumping to wrong project

## Ready Checklist

Change status to **Ready** when:

- [ ] Purpose and scope are clear
- [ ] Fuzzy matching algorithm decided (see Open Questions in index)
- [ ] Similarity threshold default chosen
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
