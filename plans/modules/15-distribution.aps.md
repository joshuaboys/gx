# Distribution & Install UX

| Field | Value |
|-------|-------|
| Module ID | DISTRIBUTION |
| Status | Draft |
| Version | v5 |
| Owner | @joshuaboys |
| Depends On | CLI |

## Purpose

Make `gx` trivial to install and upgrade for first-time users.

## Outcomes

- Prebuilt binaries published on GitHub Releases for Linux/macOS
- Single-command install script
- `gx doctor` verifies runtime, shell plugin, config paths

## Work Items

### DIST-001: Release artifacts
- Build and upload precompiled binaries in CI
- Publish checksums and release notes

### DIST-002: One-line installer
- `curl | bash` installer with platform detection
- Safe fallback instructions when unsupported

### DIST-003: Doctor command
- Validate PATH, plugin wiring, config/index health
- Return actionable fix suggestions
