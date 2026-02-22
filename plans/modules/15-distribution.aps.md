# Distribution & Install UX

| ID | Owner | Status |
|----|-------|--------|
| DIST | @joshuaboys | Draft |

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

## Ready Checklist

- [ ] Purpose and scope are clear
- [ ] Dependencies identified
- [ ] At least one task defined

## Tasks

*No tasks yet — module is Draft*
