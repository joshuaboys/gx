# URL Parsing & Path Mapping

| ID | Owner | Status |
|----|-------|--------|
| URL | @joshuaboys | Draft |

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

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

*No tasks yet — module is Draft*

## Execution *(optional)*

Steps: [../execution/URL.steps.md](../execution/URL.steps.md)
