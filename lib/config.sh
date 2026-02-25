#!/usr/bin/env bash
#
# APS runtime config loader (.apsrc.yaml)
#

aps_config_reset_defaults() {
  # Effective runtime defaults (safe, backward-compatible)
  APS_CFG_VERSION="1"
  APS_CFG_MODE_PLANNING="native"
  APS_CFG_MODE_EXECUTION="gated"
  APS_CFG_MODE_VERIFICATION="strict"
  APS_CFG_ADAPTERS_ENABLE_FALLBACK="true"
  APS_CFG_ADAPTERS_REQUIRE_NORMALIZATION="true"
  APS_CFG_GATES_REQUIRE_DESIGN_DRILL="true"
  APS_CFG_GATES_REQUIRE_PROOF_PROGRESS="true"
  APS_CFG_GATES_REQUIRE_DOD_DONE="true"
  APS_CFG_GATES_BLOCKER_ESCALATION_MINUTES="5"
  APS_CFG_PROFILE="startup-ceo"
  APS_CFG_SOURCE="defaults"
}

# Initialize defaults at file load
aps_config_reset_defaults

# Parse a simple YAML key by exact indentation context and key name.
# Example: yaml_get "$file" "mode" "verification"
yaml_get() {
  local file="$1"
  local section="$2"
  local key="$3"

  awk -v section="$section" -v key="$key" '
    BEGIN { in_section=0 }
    $0 ~ "^" section ":" { in_section=1; next }
    in_section && $0 ~ "^[A-Za-z0-9_-]+:" { in_section=0 }
    in_section {
      pattern="^[[:space:]]+" key ":[[:space:]]*"
      if ($0 ~ pattern) {
        sub(pattern, "", $0)
        if ($0 ~ /^[[:space:]]*#/) {
          $0 = ""
        } else {
          gsub(/[[:space:]]+#.*/, "", $0)
        }
        gsub(/^"|"$/, "", $0)
        gsub(/^\047|\047$/, "", $0)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", $0)
        print $0
        exit
      }
    }
  ' "$file"
}

load_aps_config() {
  local root="${1:-.}"
  local file="$root/.apsrc.yaml"

  # reset to defaults on each load
  aps_config_reset_defaults

  if [[ ! -f "$file" ]]; then
    return 0
  fi

  if [[ ! -r "$file" ]]; then
    echo "warning: APS config file '$file' exists but is not readable; using defaults" >&2
    return 0
  fi

  local v

  v=$(grep -E '^version:[[:space:]]*' "$file" | head -1 | sed -E 's/^version:[[:space:]]*//' | sed -E 's/[[:space:]]+#.*$//' | tr -d '"\047' | xargs || true)
  [[ -n "$v" ]] && APS_CFG_VERSION="$v"

  v=$(yaml_get "$file" "mode" "planning" || true)
  [[ -n "$v" ]] && APS_CFG_MODE_PLANNING="$v"

  v=$(yaml_get "$file" "mode" "execution" || true)
  [[ -n "$v" ]] && APS_CFG_MODE_EXECUTION="$v"

  v=$(yaml_get "$file" "mode" "verification" || true)
  [[ -n "$v" ]] && APS_CFG_MODE_VERIFICATION="$v"

  v=$(yaml_get "$file" "adapters" "enableFallbackAdapters" || true)
  [[ -n "$v" ]] && APS_CFG_ADAPTERS_ENABLE_FALLBACK="$v"

  v=$(yaml_get "$file" "adapters" "requireSchemaNormalization" || true)
  [[ -n "$v" ]] && APS_CFG_ADAPTERS_REQUIRE_NORMALIZATION="$v"

  v=$(yaml_get "$file" "gates" "requireDesignDrill" || true)
  [[ -n "$v" ]] && APS_CFG_GATES_REQUIRE_DESIGN_DRILL="$v"

  v=$(yaml_get "$file" "gates" "requireProofForProgress" || true)
  [[ -n "$v" ]] && APS_CFG_GATES_REQUIRE_PROOF_PROGRESS="$v"

  v=$(yaml_get "$file" "gates" "requireDodForDone" || true)
  [[ -n "$v" ]] && APS_CFG_GATES_REQUIRE_DOD_DONE="$v"

  v=$(yaml_get "$file" "gates" "blockerEscalationMinutes" || true)
  [[ -n "$v" ]] && APS_CFG_GATES_BLOCKER_ESCALATION_MINUTES="$v"

  v=$(yaml_get "$file" "init" "profile" || true)
  [[ -n "$v" ]] && APS_CFG_PROFILE="$v"

  APS_CFG_SOURCE="$file"
}

show_aps_config() {
  cat <<EOF
APS config
source: $APS_CFG_SOURCE
version: $APS_CFG_VERSION
mode.planning: $APS_CFG_MODE_PLANNING
mode.execution: $APS_CFG_MODE_EXECUTION
mode.verification: $APS_CFG_MODE_VERIFICATION
adapters.enableFallbackAdapters: $APS_CFG_ADAPTERS_ENABLE_FALLBACK
adapters.requireSchemaNormalization: $APS_CFG_ADAPTERS_REQUIRE_NORMALIZATION
gates.requireDesignDrill: $APS_CFG_GATES_REQUIRE_DESIGN_DRILL
gates.requireProofForProgress: $APS_CFG_GATES_REQUIRE_PROOF_PROGRESS
gates.requireDodForDone: $APS_CFG_GATES_REQUIRE_DOD_DONE
gates.blockerEscalationMinutes: $APS_CFG_GATES_BLOCKER_ESCALATION_MINUTES
init.profile: $APS_CFG_PROFILE
EOF
}
