<!-- Archived: 2026-02-24 | Reason: All work items complete -->

# URL Parsing & Path Mapping

| ID | Owner | Status |
|----|-------|--------|
| URL | @joshuaboys | Complete |

## Purpose

Parse git repository URLs in all common formats and convert them to consistent filesystem paths. This is the foundation that clone and index modules depend on.

## In Scope

- Parse HTTPS, SSH, git://, and shorthand (user/repo) URLs
- Normalize URLs to a canonical form (host/user/repo)
- Map URLs to filesystem paths based on config (flat vs host-prefixed)
- Validate and sanitize URLs against path traversal and injection
- Expand shorthand `user/repo` to `https://github.com/user/repo`

## Out of Scope

- Actually cloning repositories (that's Clone module)
- Local path resolution or index lookups

## Interfaces

**Depends on:**

- Config — to read `defaultHost` and `structure` settings

**Exposes:**

- `parseUrl(input: string): ParsedRepo` — parse any git URL format
- `toPath(parsed: ParsedRepo, config: Config): string` — convert to filesystem path
- `toCloneUrl(parsed: ParsedRepo): string` — convert to cloneable HTTPS/SSH URL

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

### URL-001: Implement URL parser for all git formats

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Parse HTTPS, SSH, git://, and shorthand URLs into a consistent `ParsedRepo` structure
- **Expected Outcome:** `parseUrl()` correctly extracts host, user, and repo from all supported formats
- **Validation:** `bun test tests/lib/url.test.ts`
- **Files:** `src/lib/url.ts`, `src/types.ts`, `tests/lib/url.test.ts`

### URL-002: Implement path mapping from parsed URLs

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Convert parsed repo info to filesystem paths supporting both flat and host-prefixed structures
- **Expected Outcome:** `toPath()` produces correct directory paths based on config structure setting
- **Validation:** `bun test tests/lib/path.test.ts`
- **Files:** `src/lib/path.ts`, `tests/lib/path.test.ts`
