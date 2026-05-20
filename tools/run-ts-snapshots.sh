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

if [[ "${1:-}" == "--update" ]]; then
  export INSTA_UPDATE=always
else
  export INSTA_UPDATE=no
fi

cargo test -p gx --test snapshots -- --ignored --test-threads=1
