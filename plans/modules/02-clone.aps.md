# Clone

| ID | Owner | Status |
|----|-------|--------|
| CLN | @joshuaboys | Draft |

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

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Work Items

*No tasks yet — module is Draft*

## Execution *(optional)*

Steps: [../execution/CLN.steps.md](../execution/CLN.steps.md)
