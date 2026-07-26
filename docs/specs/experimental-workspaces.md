# Experimental Workspaces

| Field | Value |
| --- | --- |
| Status | Proposed |
| Date | 2026-07-26 |
| Owner | @joshuaboys |
| Related | `plans/modules/22-experimental-workspaces.aps.md`, `plans/modules/13-tui.aps.md`, `plans/modules/09-tracking.aps.md` |
| Inspiration | [tobi/try](https://github.com/tobi/try) |

## Summary

Experimental Workspaces extends `gx` from a Git project manager into a workspace lifecycle manager.

Today, `gx` is optimised for permanent repositories: clone them into a structured hierarchy, index them, and return to them quickly. Developers also create short-lived experiments, spikes, throwaway clones and worktrees. Those workspaces commonly end up under `/tmp`, on the desktop, or in vaguely named directories that are difficult to find, assess or clean up later.

The feature introduces `gx try`, a deliberately lightweight surface for creating, finding, promoting and safely removing experimental workspaces. Experiments remain part of the same `gx` index and navigation model as permanent projects, but carry explicit lifecycle metadata and live in a separately configured directory.

The intended user journey is:

> Start an idea with almost no ceremony, find it again instantly, and promote it without losing context if it becomes real.

## Goals

- Create a named experiment and enter it with one command.
- Support empty directories, cloned repositories and worktrees.
- Keep experiments organised with predictable, date-prefixed names.
- Make experiments discoverable through fuzzy search and recorded recency.
- Allow an experiment to become a permanent project without copying files.
- Provide guarded clean-up that understands Git and worktree state.
- Reuse `gx` configuration, indexing, shell integration, URL parsing, cloning and tracking.
- Preserve compatibility with existing configuration and index files.
- Work consistently across supported shells and operating systems.

## Non-goals

- A general-purpose temporary-file manager.
- Automatic deletion based only on age.
- Running arbitrary project hooks when an experiment is entered.
- Replacing dedicated Git worktree tools such as Worktrunk.
- Synchronising experiment content to a hosted service.
- Treating experiments as lower-quality or ungoverned projects.
- Adding a separate experiment database or independent navigation system.

## Terminology

| Term | Meaning |
| --- | --- |
| Project | A permanent, indexed workspace in the configured project hierarchy. |
| Experiment | A workspace created under the configured experiment directory and intended to be provisional. It may or may not be a Git repository. |
| Experimental worktree | An experiment backed by a Git worktree from another indexed or local repository. |
| Promotion | Moving an experiment into the permanent project hierarchy and changing its indexed kind without copying its contents. |
| Clean-up | Explicit, guarded removal of one or more experiments. |

`gx try` is the user-facing command. Internally, the domain term is `experiment` so configuration and data structures remain unambiguous.

## User Experience

### Create an empty experiment

```sh
gx try redis-pool
```

Creates and enters:

```text
~/Projects/tries/2026-07-26-redis-pool
```

If the name already exists, `gx` selects the next available name:

```text
2026-07-26-redis-pool-2
2026-07-26-redis-pool-3
```

### Browse experiments

```sh
gx try
```

Opens an interactive experiment selector. Entries are filtered as the user types and ordered using fuzzy relevance with a bounded recency boost.

The selector supports:

- Arrow keys or `Ctrl+P` and `Ctrl+N` to navigate.
- Enter to select.
- `/` or direct typing to filter.
- `r` to rename.
- `p` to promote.
- Space to mark for clean-up.
- Escape or `q` to exit.

An unmatched, non-empty query offers to create a new experiment with that name.

### Clone into an experiment

```sh
gx try clone tobi/try
gx try clone https://github.com/tobi/try.git ruby-cli
```

The existing `gx` URL parser, clone behaviour and authentication environment are reused. With no explicit name, the default includes the repository owner and name:

```text
2026-07-26-tobi-try
```

### Create an experimental worktree

```sh
gx try . auth-spike
gx try worktree auth-spike
gx try worktree gx auth-spike
```

The first two forms use the current repository. The third resolves `gx` through the index.

The implementation must create the worktree through a Git-aware backend. It may use Worktrunk when configured and available, with native `git worktree` as the portable baseline. The resulting index entry records its source repository so later promotion and clean-up use worktree-aware operations.

An experimental worktree must not leave commits reachable only through a detached `HEAD`. The backend must either create a named branch or prove that the checked-out commit remains reachable before destructive clean-up.

### Jump directly to an experiment

```sh
gx try redis
```

Resolution order:

1. Exact experiment name or alias.
2. High-confidence fuzzy match.
3. Interactive disambiguation when several matches remain.
4. Offer to create a new experiment when there is no suitable match.

The existing global command remains unchanged:

```sh
gx redis
```

It may resolve any indexed workspace, including an experiment, subject to the normal ambiguity rules.

### Rename an experiment

```sh
gx try rename redis-pool redis-cache
```

Rename updates the directory and index atomically from the user's perspective. Date prefixes are retained unless the user supplies an explicitly dated name. Git worktrees are moved through `git worktree move` or the selected worktree backend.

### Promote an experiment

```sh
gx try promote redis-pool
gx try promote redis-pool --owner joshuaboys
```

Promotion:

1. Resolves the experiment and checks the destination.
2. Removes the date prefix from the proposed permanent name.
3. Previews the source, destination and any Git implications.
4. Moves the directory using a worktree-aware operation when required.
5. Changes the index entry from `experiment` or `worktree` to `project`.
6. Preserves creation and visit history.
7. Enters the promoted project.

No compatibility symlink is left in the experiment directory. The index is the source of discovery, so retaining a symlink would create duplicate paths and complicate clean-up.

### Clean up experiments

```sh
gx try clean
gx try clean --older-than 30d
gx try clean redis-pool
```

Age filters select candidates; they never authorise deletion by themselves. The user must confirm the final set.

Before removal, `gx` classifies every candidate:

| State | Default action |
| --- | --- |
| Empty or non-Git experiment | Eligible after confirmation. |
| Clean Git repository with no unique local commits | Eligible after confirmation. |
| Dirty repository or untracked files | Refuse and explain. |
| Branch with commits not present on a configured remote | Refuse and explain. |
| Detached commits that may become unreachable | Refuse and explain. |
| Registered worktree | Remove through the Git-aware backend. |
| Promoted project or ordinary indexed project | Never eligible. |
| Path outside the configured experiment directory | Never eligible. |

`--force` may bypass Git-state refusals only after an additional explicit confirmation. It must not bypass path containment or workspace-kind checks.

Recoverable operating-system deletion should be preferred where supported. If only permanent deletion is available, the confirmation must state that clearly.

## Workspace Model

The current index is extended rather than replaced.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceKind {
    Project,
    Experiment,
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexEntry {
    pub path: String,
    pub url: String,
    pub cloned_at: String,
    pub last_visited: Option<String>,
    pub kind: Option<WorkspaceKind>,
    pub created_at: Option<String>,
    pub source_project: Option<String>,
    pub disposable: Option<bool>,
}
```

Compatibility rules:

- An entry without `kind` is a `project`.
- `createdAt` is the canonical creation time for new workspaces.
- Existing entries fall back to `clonedAt` when `createdAt` is absent.
- `sourceProject` is present only when lifecycle operations need to find an originating repository.
- Optional fields are omitted when not applicable, preserving readable JSON and avoiding a mandatory migration.
- Existing `gx` commands continue to accept the old schema.

The index may retain its current name key for the first implementation. Any collision must be surfaced rather than silently overwriting another workspace. A future schema migration may introduce stable workspace IDs if rename and cross-root collision handling prove that necessary.

## Configuration

Add one backwards-compatible field:

```json
{
  "projectDir": "~/Projects/src",
  "experimentDir": "~/Projects/tries"
}
```

`experimentDir` defaults to `~/Projects/tries`. The value can be changed with:

```sh
gx config set experimentDir ~/code/experiments
```

The experiment directory is independent of the permanent project structure setting. Experiments always use a flat, date-prefixed layout because they are provisional and are primarily found through `gx`, not by traversing owner directories.

## Naming

- Generated directory format: `YYYY-MM-DD-<slug>`.
- Slugs accept letters, numbers, `_`, `-` and `.`.
- Whitespace is normalised to `-`.
- Path separators, `.` and `..` segments, control characters and empty names are rejected.
- A collision with a trailing number increments that number.
- Other collisions append `-2`, then increment.
- Display may omit the date prefix, but stored names and paths remain exact.

## Discovery and Ranking

The current Jaro-Winkler implementation remains the textual similarity source. Experiment ranking adds a bounded recency adjustment derived from indexed `lastVisited`; it must not rely on directory modification time.

Exact matches always win. Recency must never make a weak textual match auto-jump to an unrelated workspace. The existing similarity and auto-jump thresholds remain safety boundaries until benchmarks justify changing them.

The ranking function must be deterministic for a fixed query and index state and covered by table-driven tests.

## TUI Integration

Experimental Workspaces should not create an unrelated TUI architecture.

- `gx try` launches the generic selector filtered to experiment and experimental-worktree kinds.
- `gx tui` shows all workspace kinds.
- Dashboard data provides Git state when available.
- Tracking provides recency.
- Lifecycle actions are enabled according to workspace kind.

The TUI module should be revised so selection, filtering and lifecycle actions operate on a shared `WorkspaceView` model. Git details may load progressively, but navigation and exit must never block on a slow repository.

## Shell Contract

Like `gx clone` and `gx resolve`, successful commands that change directory emit the final path to stdout for the shell wrapper. Diagnostics, previews and warnings go to stderr.

Shell integration must support:

- zsh
- bash
- fish
- PowerShell once the Windows module lands

All emitted paths must use shell-appropriate quoting. The core command must never assume a POSIX shell when constructing lifecycle operations.

## Failure and Recovery

Operations that change both the filesystem and index use a recoverable sequence:

1. Validate all paths and Git preconditions.
2. Calculate the intended index mutation without saving it.
3. Perform the filesystem or worktree operation.
4. Save the index atomically.
5. If the index save fails, attempt to move the workspace back and report both the primary and rollback outcomes.

Clean-up is the exception because deletion may not be reversible. The index entry is removed only after deletion succeeds. If deletion succeeds and index persistence fails, `gx doctor` must be able to identify and prune the missing entry.

No operation silently overwrites an existing destination.

## Security and Safety Invariants

- Canonicalise source, destination and configured roots before mutation.
- Never remove a path unless it is contained by the canonical experiment root.
- Never follow a symlink to authorise deletion outside the experiment root.
- Never treat age as deletion approval.
- Never delete an ordinary project through `gx try clean`.
- Never discard dirty files or unique commits without an explicit force path.
- Never interpolate user-controlled paths into an unevaluated shell string.
- Invoke Git and worktree tools with structured process arguments.
- Record enough context in errors for the user to recover manually.

## Observability

`gx doctor` should report:

- Whether `experimentDir` exists and is writable.
- Indexed experiments whose paths are missing.
- Directories under `experimentDir` that are not indexed.
- Worktree entries whose source repository cannot be resolved.
- Entries whose kind and physical Git state disagree.
- Promoted entries that still point inside `experimentDir`.

`gx try ls --json` should expose stable machine-readable output for agent and scripting use without requiring the TUI.

## Accessibility

- All TUI actions have visible keyboard hints.
- Colour is never the only indicator of workspace kind, selection or deletion state.
- `NO_COLOR` and non-TTY output are respected.
- Every interactive operation has a non-interactive CLI equivalent.
- Destructive confirmation is readable by screen readers and usable without function keys.

## Rollout

### Phase 1: Domain foundation

Add workspace kinds, `experimentDir`, compatible index loading and experiment indexing.

### Phase 2: Core creation

Ship empty experiment creation, direct resolution, date naming and collision handling.

### Phase 3: Git-backed experiments

Ship experimental clones and worktrees with provenance metadata.

### Phase 4: Lifecycle

Ship rename, promotion, safety inspection and clean-up.

### Phase 5: Interactive experience

Ship the shared TUI selector and lifecycle actions.

## Acceptance Scenarios

1. An existing index created by the current release loads without modification and every entry behaves as a project.
2. `gx try redis` creates a dated experiment, indexes it, records the visit and enters it.
3. Repeating the same creation produces a non-conflicting, deterministically versioned name.
4. `gx try clone tobi/try` reuses normal clone authentication and creates an experiment entry.
5. `gx try . auth-spike` creates a registered worktree that can be found from both `gx try` and global navigation.
6. Promoting a normal experiment moves it, preserves history and leaves no duplicate experiment entry.
7. Promoting a worktree uses a Git-aware move and leaves `git worktree list` consistent.
8. Clean-up refuses a dirty experiment and explains the blocking files.
9. Clean-up refuses an experimental branch with unique local commits.
10. Clean-up cannot remove a symlink target or path outside `experimentDir`.
11. A non-interactive caller can list, create, promote and inspect experiments without parsing ANSI output.
12. Existing clone, resolve, recent, resume and shell-integration snapshots remain unchanged unless deliberately extended.

## Open Questions

- Should the first release expose `gx try clean --force`, or require manual Git recovery for every unsafe state?
- Should native Git or Worktrunk be the default worktree backend when both are installed?
- Should `gx recent` include experiments by default or require `--all`?
- Is a stable workspace ID required in the first schema extension, or can name-keyed entries remain until a later migration?
- Which recoverable deletion mechanisms can be supported consistently without compromising the single-binary goal?

## Attribution

The dated experiment directory, recency-oriented selector, disposable worktree, rename, promotion and guarded clean-up concepts are inspired by [`tobi/try`](https://github.com/tobi/try), licensed under MIT. `gx` should add it to the README acknowledgements when this feature becomes user-facing.
