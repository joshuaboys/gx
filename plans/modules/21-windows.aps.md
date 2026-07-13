# Windows Support

| ID  | Owner       | Status |
| --- | ----------- | ------ |
| WIN | @joshuaboys | Ready  |

- Version: v5
- Depends on: Distribution & Install UX, Shell Portability, CLI

## Purpose

Make `gx` a first-class native Windows tool: prebuilt binaries, PowerShell
integration, a checksum-verified installer, and a `doctor` that understands
Windows conventions.

## In Scope

- Windows CI coverage (build + test on `windows-latest`)
- `x86_64-pc-windows-msvc` release binary with checksum and attestation
- PowerShell shell integration via `gx shell-init powershell` (cd wrapper + tab completion)
- Windows-aware `gx doctor` (finds `gx.exe` on PATH, resolves home without `$HOME`, checks the PowerShell profile)
- PowerShell installer (`install.ps1`, `irm | iex` flow) with checksum verification
- Windows install and setup documentation

## Out of Scope

- Package manager submissions (winget, scoop, chocolatey) — deferred, consistent with the DIST module's package-manager deferral
- cmd.exe integration — PowerShell only
- Windows ARM64 (`aarch64-pc-windows-msvc`) — until there is demand
- WSL — already served by the Linux binaries

## Interfaces

**Depends on:**

- Distribution & Install UX (DIST) — release pipeline, checksum/attestation trust model, installer conventions, `gx doctor`
- Shell Portability (SHELL) — `gx shell-init` architecture and per-shell script generation
- CLI — command surface

**Exposes:**

- `gx-windows-x64.exe` release asset (covered by `SHA256SUMS` and attestations)
- `gx shell-init powershell` integration script
- `install.ps1` installer

## Implementation Notes

Current state (verified against source, 2026-07-13):

- Platform-specific code is already gated: `#[cfg(unix)]` with `#[cfg(not(unix))]` fallbacks in `commands/doctor.rs`, `commands/open.rs`, and a unix-only symlink test in `index_store.rs`. The crate is expected to compile for `x86_64-pc-windows-msvc`; WIN-1 confirms this and fixes what falls out.
- `doctor` searches PATH for a file literally named `gx` (misses `gx.exe`) and reads `$HOME`/`$SHELL`, neither of which is set on native Windows.
- `shell_init` supports zsh/bash/fish only and detects the shell from `$SHELL`.
- Config and index live at `home_dir()/.config/gx/` via the `dirs` crate. Keep this location on Windows (`C:\Users\<user>\.config\gx\`) rather than moving to `%APPDATA%` — schema and path consistency across platforms outweighs Windows idiom, and it avoids a migration. Recorded as D-011 in the index.
- `.github/workflows/release.yml` has four unix targets and stages the binary with a bare `cp`; the Windows job must account for the `.exe` suffix. `ci.yml` does not run on `windows-latest`.
- Git Bash users get bash integration for free once the binary exists; worth a docs note, not code.

Chosen shape for the installer: `install.ps1` mirrors `install.sh` — download from GitHub Releases, verify against `SHA256SUMS`, install to a user-writable bin directory, ensure PATH, print shell-init instructions. No new trust model.

## Ready Checklist

Change status to **Ready** when:

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one work item defined
- [x] Human review of scope and installer shape (approved 2026-07-13)

## Work Items

### WIN-001: Windows CI coverage

- **Intent:** Prove the crate builds and tests green on Windows, and keep it that way.
- **Expected Outcome:** `windows-latest` job in `ci.yml` runs fmt/clippy/test; suite passes with any portability fixes applied.
- **Validation:** CI run on a PR shows the Windows job green.

### WIN-002: Windows release binary

- **Intent:** Publish a prebuilt Windows binary through the existing trust model.
- **Expected Outcome:** Tag builds upload `gx-windows-x64.exe`, included in `SHA256SUMS` and attestations.
- **Validation:** Release workflow run for a tag lists the asset; `gh attestation verify` passes against it.
- **Dependencies:** WIN-001

### WIN-003: PowerShell shell integration

- **Intent:** Give PowerShell users the same jump/cd/completion UX as zsh/bash/fish.
- **Expected Outcome:** `gx shell-init powershell` emits a working cd wrapper and tab completion; shell detection handles PowerShell.
- **Validation:** `cargo test --workspace` (shell-init snapshot tests cover the new shell)
- **Dependencies:** WIN-001

### WIN-004: Windows-aware doctor

- **Intent:** `gx doctor` reports accurate health on Windows instead of unix-shaped noise.
- **Expected Outcome:** Doctor finds `gx.exe` on PATH, resolves home without `$HOME`, and checks the PowerShell profile for shell integration.
- **Validation:** `cargo test --workspace` (doctor unit tests cover Windows paths); manual `gx doctor` run on Windows shows no false negatives.
- **Dependencies:** WIN-001, WIN-003

### WIN-005: PowerShell installer

- **Intent:** One-command install on Windows matching the unix installer's trust model.
- **Expected Outcome:** `install.ps1` downloads the release asset, verifies its checksum against `SHA256SUMS`, installs to a user bin dir, and ensures PATH.
- **Validation:** `pwsh -File install.ps1` on a clean Windows machine leaves a working `gx` on PATH.
- **Dependencies:** WIN-002

### WIN-006: Windows documentation

- **Intent:** A Windows user can install and set up gx from the README alone.
- **Expected Outcome:** README documents the installer, PowerShell setup, and Git Bash/WSL notes.
- **Validation:** README section exists and matches shipped behavior.
- **Dependencies:** WIN-003, WIN-005
