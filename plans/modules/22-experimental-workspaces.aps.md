# Experimental Workspaces

| ID | Owner | Status |
| --- | --- | --- |
| EXP | @joshuaboys | Draft |

- Version: v6
- Specification: [Experimental Workspaces](../../docs/specs/experimental-workspaces.md)
- Inspiration: [tobi/try](https://github.com/tobi/try)
- Depends on: Index, Clone, Shell Plugin, Tracking, Rust Port
- Informs: Dashboard, Interactive TUI, Index Reliability & Observability, Windows Support

## Purpose

Give provisional ideas a first-class lifecycle inside `gx`: create an experiment with almost no ceremony, find it again through normal navigation, understand its Git state, promote it when it becomes real, and remove it safely when it does not.

## Outcome

`gx` manages permanent projects and provisional experiments through one compatible workspace index. Users gain a compact `gx try` surface for empty experiments, clones and worktrees without introducing a separate database or duplicating existing clone, fuzzy, tracking and shell behaviour.

## In Scope

- `gx try [query]` creation, resolution and filtered interactive selection
- Configurable experiment root with date-prefixed, collision-safe naming
- Empty, cloned and worktree-backed experiments
- Backwards-compatible workspace-kind and lifecycle metadata in the index
- Recency-aware experiment discovery using indexed visits
- Rename and promotion into the permanent project hierarchy
- Git-aware, path-contained clean-up with explicit confirmation
- Non-interactive list and inspection output for scripting and agents
- Shared selector model that can also underpin the future all-workspace TUI
- Doctor diagnostics for missing, orphaned and inconsistent experiments
- Cross-platform command and path contracts

## Out of Scope

- Automatic age-based deletion
- Hosted experiment synchronisation
- Arbitrary commands or hooks on experiment entry
- Git commit, push, pull or merge operations from the selector
- Replacing Worktrunk or exposing every worktree-management feature
- A second index dedicated to experiments
- Compatibility symlinks after promotion

## Interfaces

**Depends on:**

- Index for compatible persistence, lookup and atomic writes
- Clone and URL parsing for repository-backed experiments
- Shell Plugin for parent-shell directory changes
- Tracking for `lastVisited`
- Rust Port as the active implementation

**Exposes:**

- `gx try [query]`
- `gx try clone <repo> [name]`
- `gx try . <name>`
- `gx try worktree [project] <name>`
- `gx try ls [--json]`
- `gx try rename <experiment> <name>`
- `gx try promote <experiment>`
- `gx try clean [experiment] [--older-than <age>]`
- Workspace kind and experiment provenance for Dashboard, TUI and Doctor

## Design Commitments

1. Existing index entries without a workspace kind remain permanent projects.
2. Experiments share the normal index, tracking and global navigation path.
3. `experimentDir` is a separate flat root and defaults to `~/Projects/tries`.
4. Filesystem modification time is not visit history; discovery uses indexed `lastVisited`.
5. Promotion moves and reclassifies the workspace without leaving a symlink.
6. Clean-up never targets ordinary projects and never relies on age as authorisation.
7. Worktree promotion and removal use Git-aware operations.
8. Interactive features always have a non-interactive equivalent.
9. User-controlled paths are passed as process arguments, never interpolated into shell commands.

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and product boundary are documented
- [x] Initial command surface is specified
- [x] Backwards-compatible index direction is specified
- [x] Safety invariants are specified
- [x] Dependencies and consumers are identified
- [x] Work items and validation outcomes are defined
- [ ] Open questions in the specification have owner decisions
- [ ] Human review accepts the command name, schema fields and clean-up policy

## Work Items

### EXP-001: Workspace kinds and experiment configuration

- **Intent:** Represent provisional workspaces without breaking existing users.
- **Expected Outcome:** Add `experimentDir` and optional workspace lifecycle fields; existing configuration and index fixtures continue to load byte-for-byte where no new data is written.
- **Validation:** Unit tests load old fixtures as `project`; config tests prove the default and custom experiment roots; snapshot tests for existing commands remain unchanged.

### EXP-002: Experiment creation and direct navigation

- **Intent:** Make starting an organised experiment a one-command operation.
- **Expected Outcome:** `gx try <name>` creates a date-prefixed directory, indexes and touches it, prints the final path for shell integration, and resolves existing experiment queries instead of duplicating them.
- **Validation:** Integration tests cover first creation, exact resolution, fuzzy resolution, ambiguous matches, invalid names and non-TTY output.
- **Dependencies:** EXP-001

### EXP-003: Deterministic naming and collision handling

- **Intent:** Preserve zero-ceremony creation without overwriting or confusing existing experiments.
- **Expected Outcome:** Names are slugged consistently; repeated names receive predictable numeric suffixes; every candidate path is contained by the experiment root.
- **Validation:** Table-driven tests cover whitespace, Unicode policy, trailing numbers, existing destinations, traversal attempts, separators and canonical-path containment.
- **Dependencies:** EXP-001

### EXP-004: Experimental clone flow

- **Intent:** Let users inspect or modify a repository provisionally without placing it among permanent projects.
- **Expected Outcome:** `gx try clone <repo> [name]` reuses normal URL parsing and Git authentication, creates the clone under `experimentDir`, and records experiment metadata and origin URL.
- **Validation:** Integration tests use local bare repositories for shorthand-derived names, explicit names, clone failure rollback and index persistence.
- **Dependencies:** EXP-001, EXP-003

### EXP-005: Experimental worktree flow

- **Intent:** Give repository spikes an isolated, disposable worktree with enough provenance for safe lifecycle operations.
- **Expected Outcome:** Current-directory and indexed-project forms create a registered worktree, record its source, avoid unreachable detached commits, and enter it through the shell contract.
- **Validation:** Tests assert `git worktree list` consistency, branch reachability, source resolution, collision handling and rollback after a failed index save.
- **Dependencies:** EXP-001, EXP-003

### EXP-006: Machine-readable list and inspection

- **Intent:** Make the lifecycle usable by scripts and coding agents without scraping the TUI.
- **Expected Outcome:** `gx try ls` provides stable human output and `gx try ls --json` exposes workspace kind, path, age, source and a Git-safety summary.
- **Validation:** Snapshot human output; schema-test JSON output; confirm colour and terminal control codes never appear in JSON.
- **Dependencies:** EXP-001

### EXP-007: Shared interactive selector

- **Intent:** Provide fast fuzzy navigation while avoiding a one-off experiment TUI.
- **Expected Outcome:** A reusable workspace selector supports filtering, recency display, responsive terminal layout and selection; `gx try` applies an experiment-kind filter and the future `gx tui` can reuse the same component.
- **Validation:** State-machine tests cover navigation, editing, resize, cancel, exact selection, create-new selection and non-colour presentation.
- **Dependencies:** EXP-002, EXP-006

### EXP-008: Rename lifecycle

- **Intent:** Let an experiment's name evolve without losing history or corrupting registered worktrees.
- **Expected Outcome:** Rename preflights the destination, performs a normal or worktree-aware move, updates the index, preserves timestamps and rolls back when persistence fails.
- **Validation:** Tests cover normal directories, Git repositories, worktrees, destination collisions and simulated index-write failure.
- **Dependencies:** EXP-003, EXP-005

### EXP-009: Promotion lifecycle

- **Intent:** Turn a successful experiment into a permanent project without copying it or leaving duplicate discovery paths.
- **Expected Outcome:** Promotion previews and moves the workspace, removes the date prefix by default, respects the configured project structure, reclassifies the index entry, preserves history and leaves no compatibility symlink.
- **Validation:** End-to-end tests cover empty directories, regular repositories, worktrees, destination collision, rollback and subsequent global resolution.
- **Dependencies:** EXP-005, EXP-008

### EXP-010: Safe clean-up engine

- **Intent:** Make abandoned experiments removable without normalising destructive repository loss.
- **Expected Outcome:** Candidate inspection classifies dirty files, untracked content, unique commits, detached reachability, worktree registration and path containment; only explicitly confirmed eligible experiments are removed.
- **Validation:** Adversarial tests cover symlinks, `..`, changed roots, dirty repos, untracked files, unpushed branches, detached commits, ordinary projects and current-working-directory removal.
- **Dependencies:** EXP-004, EXP-005, EXP-006

### EXP-011: Interactive lifecycle actions

- **Intent:** Make rename, promotion and multi-select clean-up available without sacrificing the command-line safety contracts.
- **Expected Outcome:** Selector actions delegate to the same preflight and mutation services as non-interactive commands and present complete previews and refusals.
- **Validation:** State-machine and integration tests prove the TUI cannot bypass lifecycle guards or path containment.
- **Dependencies:** EXP-007, EXP-008, EXP-009, EXP-010

### EXP-012: Doctor, documentation and attribution

- **Intent:** Make the feature diagnosable, discoverable and explicit about its inspiration.
- **Expected Outcome:** `gx doctor` reports missing, orphaned and inconsistent experiments; README and use cases cover the workflow; acknowledgements credit `tobi/try`.
- **Validation:** Doctor fixtures cover every diagnostic class; documentation examples match CLI snapshots; links and acknowledgements are present.
- **Dependencies:** EXP-006, EXP-009, EXP-010

### EXP-013: Cross-platform lifecycle verification

- **Intent:** Preserve the single-binary and multi-shell promise across lifecycle operations.
- **Expected Outcome:** Paths, process invocation, worktree operations and shell output work on Linux, macOS and Windows/PowerShell when the Windows module is available.
- **Validation:** CI runs the applicable experiment suite on all supported operating systems; shell-init snapshots cover every supported shell.
- **Dependencies:** EXP-002 through EXP-012, Windows Support

## Execution Order

1. Foundation: EXP-001, EXP-003
2. Core surface: EXP-002, EXP-004, EXP-005, EXP-006
3. Lifecycle: EXP-008, EXP-009, EXP-010
4. Interactive surface: EXP-007, EXP-011
5. Hardening and adoption: EXP-012, EXP-013

The selector may be prototyped alongside the core surface, but destructive interactive actions must not land before the shared lifecycle services and guards.

## Risks

| Risk | Impact | Mitigation |
| --- | --- | --- |
| Clean-up removes valuable work | Severe and potentially irreversible | Restrict by kind and canonical root; inspect Git state; explicit confirmation; refuse unsafe states by default |
| Worktree metadata becomes inconsistent | Git commands fail or leave orphan registrations | Use Git-aware move/remove; verify with `git worktree list`; doctor diagnostics |
| Optional index fields become an accidental migration | Existing installations fail to load or rewrite unexpectedly | Serde defaults and omission; old-fixture tests; no eager rewrite |
| Recency causes an incorrect auto-jump | User enters or mutates the wrong workspace | Exact match priority; bounded boost; retain textual auto-jump threshold |
| The experiment TUI diverges from the planned global TUI | Duplicate terminal code and inconsistent behaviour | Shared `WorkspaceView` and selector state model |
| Permanent and provisional workspaces become conceptually blurred | Users accidentally treat projects as disposable | Explicit kind, separate root, destructive actions restricted to experiments |
| Cross-platform deletion and quoting differ | Safety or reliability regression on Windows | Structured process arguments, platform tests, recoverable deletion capability detection |

## Open Questions

- [ ] EXP-Q001: Ship `--force` clean-up in the first release or defer it?
- [ ] EXP-Q002: Prefer native Git or Worktrunk when both worktree backends are available?
- [ ] EXP-Q003: Include experiments in unfiltered `gx recent` by default?
- [ ] EXP-Q004: Add stable workspace IDs now or retain name-keyed index entries initially?
- [ ] EXP-Q005: Which recoverable deletion backends preserve the single-binary distribution goal?

## Completion Criteria

- The twelve acceptance scenarios in the specification pass.
- Existing configuration and index fixtures remain compatible.
- Existing command snapshots change only where the new surface is intentionally exposed.
- No destructive path can target an ordinary project or escape `experimentDir`.
- Worktree creation, promotion and clean-up leave Git metadata consistent.
- Interactive and non-interactive operations share the same lifecycle services.
- Documentation demonstrates the complete idea-to-project journey.
