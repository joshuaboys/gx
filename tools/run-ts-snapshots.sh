#!/usr/bin/env bash
# RST-2 — run the Rust snapshot harness against the TypeScript binary.
#
# Builds ./gx via Bun, points the harness at it, and runs the ignored snapshot
# tests. This is how goldens get refreshed (with INSTA_UPDATE=auto/always) and
# how CI re-verifies that the TS binary still matches the committed goldens.
#
# Usage:
#   tools/run-ts-snapshots.sh              # verify (fail on diff)
#   tools/run-ts-snapshots.sh --update     # update goldens to match TS output

set -euo pipefail

cd "$(dirname "$0")/.."

bun run build

export GX_SNAPSHOT_BIN="$PWD/gx"

# `gx shell-init` resolves the binary via `Bun.which("gx")` (PATH lookup) and
# warns to stderr when it can't find one. Goldens are captured with gx on PATH,
# so make the freshly built binary discoverable to keep stderr clean and the
# embedded `_GX_BIN` path stable across environments (CI has no gx on PATH).
export PATH="$PWD:$PATH"

if [[ "${1:-}" == "--update" ]]; then
  export INSTA_UPDATE=always
else
  export INSTA_UPDATE=no
fi

# `--include-ignored` runs both the live read-only snapshots and any
# mutating-command snapshots still marked `#[ignore]` pending their port, so
# the TS binary is verified against every committed golden.
cargo test -p gx --test snapshots -- --include-ignored --test-threads=1
