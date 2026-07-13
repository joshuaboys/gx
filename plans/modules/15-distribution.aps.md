# Distribution & Install UX

| ID   | Owner       | Status      |
| ---- | ----------- | ----------- |
| DIST | @joshuaboys | In Progress |

- Version: v5
- Depends on: CLI

## Purpose

Make `gx` trivial to install and upgrade for first-time users.

## In Scope

- Prebuilt binaries published on GitHub Releases for Linux/macOS
- Simple, cross-platform install command that downloads a binary, verifies its checksum/signature, then installs
- Clearly documented trust model (who signs releases, how checksums are distributed, how users verify)
- `gx doctor` command to verify runtime, shell plugin, config paths

## Out of Scope

- Package manager submissions (Homebrew, AUR, etc.) — deferred to a future module
- Auto-update mechanism

## Interfaces

**Depends on:**

- CLI — command surface

**Exposes:**

- `gx doctor` — health check command (index diagnostics contributed by IDXOB module)

## Implementation Notes

`install.sh` handles installation with release checksum verification:

- Downloads prebuilt binaries from GitHub Releases with OS/arch detection
- Verifies downloaded binaries against the release `SHA256SUMS` manifest before installing
- Errors with manual Cargo build guidance if no binary is available for the detected target
- Detects and installs shell integration into `.zshrc`, `.bashrc`, or Fish config
- Ensures `~/.local/bin` is on PATH

CI pipeline (`.github/workflows/ci.yml`) runs tests and builds on both `ubuntu-latest` and `macos-latest`.

Release pipeline (`.github/workflows/release.yml`) builds the Rust binary for `linux-x64`, `linux-aarch64`, `darwin-x64`, and `darwin-aarch64`, uploads `SHA256SUMS`, and emits GitHub artifact attestations for the release assets and checksum manifest.

Trust model: the installer trusts GitHub Releases as the distribution channel, verifies each downloaded binary against the co-published `SHA256SUMS` manifest, and relies on GitHub artifact attestations from the release workflow for build provenance. Users who do not match a prebuilt target are directed to build from source with Cargo.

`gx doctor` reports installation health without mutating user state:

- Runtime and `gx` binary availability on `PATH`
- Config, project directory, and index paths
- Indexed project count plus stale path count
- Shell integration presence for zsh, bash, or fish

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

- [x] DIST-1: Create `install.sh` curl installer with OS/arch detection and source-build guidance
- [x] DIST-2: CI pipeline for Linux and macOS builds
- [x] DIST-3: Implement `gx doctor` health check command
- [x] DIST-4: Release checksum verification (SHA256SUMS) and GitHub build provenance attestations
