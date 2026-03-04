# Fork & Sync

| ID  | Owner       | Status |
| --- | ----------- | ------ |
| FRK | @joshuaboys | Draft  |

## Purpose

Enable users to fork repositories and keep forks in sync with upstream via `gx fork` and `gx sync`. Forking is a first-class GitHub workflow, and gx should handle the full lifecycle: fork on the remote, clone locally, index the project, track the upstream relationship, and provide a simple way to pull upstream changes — even when the fork has diverged.

## In Scope

- `gx fork <repo>` — fork on GitHub via `gh`, clone the fork locally, set `upstream` remote, index the project, cd into it
- `gx fork <repo> --clone-only` — skip the GitHub fork (repo already forked), just clone your fork and set upstream
- `gx sync` — fetch upstream and integrate changes into current branch
- `gx sync` detects fork state and acts accordingly:
  - **Behind upstream:** fast-forward merge (safe, automatic)
  - **Ahead of upstream:** nothing to sync — upstream has no new changes, inform user
  - **Diverged (ahead and behind):** fetch upstream, then rebase or merge based on user preference (default: rebase, configurable)
- `gx sync --rebase` / `gx sync --merge` — override the default strategy
- `gx sync --push` — push to origin after syncing (update your remote fork)
- Detect if `gh` CLI is available and authenticated before forking
- Tab completion for `gx fork` and `gx sync`

## Out of Scope

- Forking on non-GitHub hosts (GitLab, Bitbucket) — `gh` is GitHub-only
- Syncing forks that have multiple upstreams or complex remote topologies
- Automatic periodic sync (cron, background daemon)
- Pull request creation from fork (`gh pr create` already handles this)
- Renaming the fork on GitHub
- Managing fork settings (e.g., disabling issues on the fork)

## Interfaces

**Depends on:**

- Clone (02) — reuses URL parsing, `git clone`, path computation, and indexing
- Shell Plugin (05) — shell wrapper for `cd` after fork, completion entries
- CLI (04) — subcommand registration

**Exposes:**

- `forkRepo(input: string, config: Config, indexPath: string, opts?: ForkOptions): Promise<string>` — fork, clone, index, return local path
- `syncFork(opts?: SyncOptions): Promise<SyncResult>` — fetch upstream, integrate changes, return status
- `fork` subcommand in CLI
- `sync` subcommand in CLI

**Types:**

```typescript
interface ForkOptions {
  cloneOnly?: boolean; // skip gh fork, just clone + set upstream
}

interface SyncOptions {
  strategy?: "rebase" | "merge"; // default: rebase
  push?: boolean; // push to origin after sync
}

interface SyncResult {
  status: "up-to-date" | "fast-forwarded" | "rebased" | "merged" | "conflict";
  ahead: number; // commits ahead of upstream
  behind: number; // commits behind upstream
}
```

## Constraints

- Requires `gh` CLI installed and authenticated — `gx fork` must check this upfront and give a clear error if missing
- `gx sync` must never silently discard user commits — diverged state uses the configured strategy (default: rebase), which can be overridden per-run with `--rebase` / `--merge`; the chosen strategy must be displayed to the user before execution
- `gx sync` must abort cleanly on rebase/merge conflicts and leave the working tree in a resolvable state (same as `git rebase` / `git merge` would)
- `gx sync` only operates on the current branch — does not sync all branches
- `upstream` remote name is a convention — `gx sync` should detect the upstream remote by checking `gh repo view --json parent` or falling back to a remote named `upstream`
- Must not break the existing `gx clone` workflow — `gx fork` is an additive command

## Ready Checklist

Change status to **Ready** when:

- [ ] `gh` CLI detection and error messaging designed
- [ ] Fork-then-clone flow validated (gh fork → parse fork URL → clone → set upstream → index)
- [ ] Sync strategy for all three states defined (behind, ahead, diverged)
- [ ] Shell integration (cd + completions) planned
- [ ] Dependencies identified
- [ ] At least one work item defined

## Work Items

_None yet -- tasks will be defined when this module reaches Ready status._
