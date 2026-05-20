# Snapshot Harness Fixtures

Canonical `~/.config/gx/` state used by the snapshot harness in
`crates/gx/tests/snapshots.rs`.

The harness copies these into an isolated `HOME` per test, runs the binary
under test, and compares stdout/stderr/exit against the goldens in
`crates/gx/tests/snapshots/`.

- `index.json` — three projects with deterministic, far-past timestamps so
  relative-time strings stay stable (the harness still scrubs them via insta
  filters as a belt-and-braces).
- `config.json` — default config values matching `DEFAULT_CONFIG` in
  `src/types.ts`.

Paths and URLs are synthetic. They never need to exist on disk because the
harness only exercises read-only commands in RST-2.
