<!-- Archived: 2026-02-24 | Reason: All work items complete -->

# Clone

| ID | Owner | Status |
|----|-------|--------|
| CLN | @joshuaboys | Complete |

## Purpose

Clone git repositories into organized directory structures, creating parent directories as needed and outputting the final path for shell integration.

## In Scope

- Shell out to `git clone` with the resolved URL and target path
- Create parent directories with safe permissions
- Detect and skip already-cloned repos
- Support shallow clones (`--depth=1`)
- Output the cloned path to stdout for the zsh plugin to `cd` into
- Update the project index after successful clone

## Out of Scope

- Parallel cloning of multiple repos (future enhancement)
- Pull/fetch operations on existing repos

## Interfaces

**Depends on:**

- URL — to parse input and determine target path
- Index — to register cloned project
- Config — to read `projectDir` and `shallow` settings

**Exposes:**

- `clone(input: string, options?: CloneOptions): Promise<string>` — clone repo, return path

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

### CLN-001: Implement clone command with directory creation and index update

| Field | Value |
|-------|-------|
| Status | Complete: 2026-02-23 |
| Confidence | high |

- **Intent:** Clone repositories to organized paths, creating directories as needed, skipping duplicates, and registering in the project index
- **Expected Outcome:** `gx clone user/repo` clones to the correct directory, skips already-cloned repos, and updates the index
- **Validation:** `bun test tests/commands/clone.test.ts`
- **Files:** `src/commands/clone.ts`, `tests/commands/clone.test.ts`
