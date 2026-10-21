#!/usr/bin/env bash
# Called by the release workflow: stage assets, attest, then publish.
set -euo pipefail

: "$GITHUB_REF_NAME" "$GITHUB_REPOSITORY"
assets=(
  dist/gx-linux-x64 dist/gx-linux-aarch64
  dist/gx-darwin-x64 dist/gx-darwin-aarch64
  dist/gx-windows-x64.exe dist/SHA256SUMS
)

require_draft() {
  local draft
  draft=$(gh release view "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" --json isDraft --jq '.isDraft')
  if [[ "$draft" != true ]]; then
    echo "Refusing to modify published release $GITHUB_REF_NAME" >&2
    return 1
  fi
}

verify_assets() (
  local verification_dir
  verification_dir=$(mktemp -d)
  trap 'rm -rf "$verification_dir"' EXIT
  # Ensure the published set would contain exactly the supported platform set.
  printf '%s\n' "${assets[@]##*/}" | LC_ALL=C sort > "$verification_dir/expected"
  gh release view "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" \
    --json assets --jq '.assets[].name' | LC_ALL=C sort > "$verification_dir/actual"
  cmp "$verification_dir/expected" "$verification_dir/actual"
  gh release download "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" \
    --dir "$verification_dir" --pattern 'gx-*' --pattern SHA256SUMS
  for asset in "${assets[@]}"; do
    cmp "$asset" "$verification_dir/$(basename "$asset")"
  done
  (cd "$verification_dir" && sha256sum --check SHA256SUMS)
)

case "${1:-}" in
  stage)
    for asset in "${assets[@]}"; do test -s "$asset"; done
    (cd dist && sha256sum --check SHA256SUMS)
    if ! gh release view "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" >/dev/null 2>&1; then
      gh release create "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" --draft --generate-notes
    fi
    require_draft
    gh release upload "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" "${assets[@]}" --clobber
    verify_assets
    ;;
  publish)
    require_draft
    verify_assets
    gh release edit "$GITHUB_REF_NAME" --repo "$GITHUB_REPOSITORY" --draft=false
    ;;
  *)
    echo "Usage: $0 stage|publish" >&2
    exit 2
    ;;
esac
