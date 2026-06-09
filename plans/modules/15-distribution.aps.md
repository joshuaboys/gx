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

`install.sh` handles basic installation (checksum/signature verification is not yet implemented — see DIST-4):

- Downloads prebuilt binaries from GitHub Releases with OS/arch detection
- Falls back to building from source via bun if no binary available
- Detects and installs shell integration into `.zshrc`, `.bashrc`, or Fish config
- Ensures `~/.local/bin` is on PATH

CI pipeline (`.github/workflows/ci.yml`) runs tests and builds on both `ubuntu-latest` and `macos-latest`.

## Ready Checklist

- [x] Purpose and scope are clear
- [x] Dependencies identified
- [x] At least one task defined

## Work Items

- [x] DIST-1: Create `install.sh` curl installer with OS/arch detection and source fallback
- [x] DIST-2: CI pipeline for Linux and macOS builds
- [ ] DIST-3: Implement `gx doctor` health check command
- [ ] DIST-4: Release signing and checksum verification
