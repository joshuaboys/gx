#!/usr/bin/env bash
#
# Scaffold logic for `aps init` and `aps update`
#

APS_VERSION="${APS_VERSION:-main}"
APS_BASE_URL="https://raw.githubusercontent.com/EddaCraft/anvil-plan-spec/$APS_VERSION"

# Files to download for plans/
PLAN_FILES=(
  "scaffold/plans/aps-rules.md"
  "scaffold/plans/modules/.module.template.md"
  "scaffold/plans/modules/.simple.template.md"
  "scaffold/plans/modules/.index-monorepo.template.md"
  "scaffold/plans/execution/.steps.template.md"
)

# Files to download for the planning skill
SKILL_FILES=(
  "scaffold/aps-planning/SKILL.md"
  "scaffold/aps-planning/reference.md"
  "scaffold/aps-planning/examples.md"
  "scaffold/aps-planning/hooks.md"
  "scaffold/aps-planning/scripts/install-hooks.sh"
  "scaffold/aps-planning/scripts/init-session.sh"
  "scaffold/aps-planning/scripts/check-complete.sh"
  "scaffold/aps-planning/scripts/pre-tool-check.sh"
  "scaffold/aps-planning/scripts/post-tool-nudge.sh"
  "scaffold/aps-planning/scripts/enforce-plan-update.sh"
  "scaffold/aps-planning/scripts/install-hooks.ps1"
  "scaffold/aps-planning/scripts/init-session.ps1"
  "scaffold/aps-planning/scripts/check-complete.ps1"
  "scaffold/aps-planning/scripts/pre-tool-check.ps1"
  "scaffold/aps-planning/scripts/post-tool-nudge.ps1"
  "scaffold/aps-planning/scripts/enforce-plan-update.ps1"
)

# Files to download for slash commands
COMMAND_FILES=(
  "scaffold/commands/plan.md"
  "scaffold/commands/plan-status.md"
)

# CLI files (bin/ and lib/)
CLI_FILES=(
  "bin/aps"
  "bin/aps.ps1"
  "lib/output.sh"
  "lib/Output.psm1"
  "lib/lint.sh"
  "lib/Lint.psm1"
  "lib/scaffold.sh"
  "lib/Scaffold.psm1"
  "lib/rules/common.sh"
  "lib/rules/Common.psm1"
  "lib/rules/module.sh"
  "lib/rules/Module.psm1"
  "lib/rules/index.sh"
  "lib/rules/Index.psm1"
  "lib/rules/workitem.sh"
  "lib/rules/WorkItem.psm1"
  "lib/rules/issues.sh"
  "lib/rules/Issues.psm1"
)

# Download a file from GitHub
download() {
  local src="$1"
  local dest="$2"
  local url="$APS_BASE_URL/$src"

  mkdir -p "$(dirname "$dest")"
  if ! curl -fsSL "$url" -o "$dest"; then
    error "Failed to download: $url"
    echo "  Check your network and ensure APS_VERSION='$APS_VERSION' is valid." >&2
    exit 1
  fi
}

# Prompt user with a yes/no question. Returns 0 for yes, 1 for no.
# Non-interactive defaults to the provided default.
ask_yn() {
  local prompt="$1"
  local default="${2:-n}"

  if [[ -t 0 ]]; then
    local yn_hint
    if [[ "$default" == "y" ]]; then yn_hint="Y/n"; else yn_hint="y/N"; fi
    printf "%s [%s] " "$prompt" "$yn_hint"
    read -r answer
    answer="${answer:-$default}"
    [[ "$answer" =~ ^[Yy] ]]
  else
    [[ "$default" == "y" ]]
  fi
}

# Prompt user to select one value from a list. Non-interactive returns default.
ask_choice() {
  local prompt="$1"
  local default="$2"
  shift 2
  local options=("$@")

  if [[ -t 0 ]]; then
    local hint
    hint="$(IFS='/'; echo "${options[*]}")"
    printf "%s [%s] (default: %s) " "$prompt" "$hint" "$default" >&2
    read -r answer
    answer="${answer:-$default}"

    local opt
    for opt in "${options[@]}"; do
      if [[ "$answer" == "$opt" ]]; then
        echo "$answer"
        return 0
      fi
    done

    echo "$default"
    return 0
  fi

  echo "$default"
}

write_apsrc_config() {
  local target="$1"
  local config_file="$target/.apsrc.yaml"

  if [[ -f "$config_file" ]]; then
    info ".apsrc.yaml already exists (left unchanged)"
    return 0
  fi

  local profile
  local verification
  local fallback_adapters="true"

  profile="$(ask_choice "Select APS profile" "startup-ceo" "startup-ceo" "enterprise" "solo-builder" "custom")"
  verification="$(ask_choice "Verification strictness" "strict" "strict" "standard")"

  if ask_yn "Enable fallback adapters when native capability is missing?" "y"; then
    fallback_adapters="true"
  else
    fallback_adapters="false"
  fi

  cat > "$config_file" <<EOF
version: 1

mode:
  planning: native
  execution: gated
  verification: $verification

adapters:
  enableFallbackAdapters: $fallback_adapters
  requireSchemaNormalization: true

gates:
  requireDesignDrill: true
  requireProofForProgress: true
  requireDodForDone: true
  blockerEscalationMinutes: 5

artifacts:
  plansDir: plans
  templatesDir: plans/templates
  statusSchema: aps-v1

init:
  askOnBootstrap: true
  profile: $profile
EOF

  info "Generated .apsrc.yaml (project APS policy)"
}

# Check if APS hooks are already configured
has_aps_hooks() {
  local target="${1:-.}"
  local settings="$target/.claude/settings.local.json"
  [[ -f "$settings" ]] && grep -q 'aps-planning/scripts\|\[APS\]' "$settings" 2>/dev/null
}

# Download plan templates to target
install_plans() {
  local target="$1"
  local plans_dir="$target/plans"

  mkdir -p "$plans_dir/modules" "$plans_dir/execution" "$plans_dir/decisions"

  for f in "${PLAN_FILES[@]}"; do
    # Strip "scaffold/plans/" prefix → destination under plans/
    local rel="${f#scaffold/plans/}"
    download "$f" "$plans_dir/$rel"
  done
}

# Download the index template (init only, not update)
install_index() {
  local target="$1"
  download "scaffold/plans/index.aps.md" "$target/plans/index.aps.md"
  touch "$target/plans/decisions/.gitkeep"
}

# Download skill files to target
install_skill() {
  local target="$1"

  for f in "${SKILL_FILES[@]}"; do
    # Strip "scaffold/" prefix → destination under aps-planning/
    local rel="${f#scaffold/}"
    download "$f" "$target/$rel"
  done
  chmod +x "$target/aps-planning/scripts/"*.sh
}

# Download slash commands to .claude/commands/
install_commands() {
  local target="$1"
  local commands_dir="$target/.claude/commands"

  mkdir -p "$commands_dir"
  for f in "${COMMAND_FILES[@]}"; do
    # Strip "scaffold/commands/" prefix
    local rel="${f#scaffold/commands/}"
    download "$f" "$commands_dir/$rel"
  done
}

# Download the CLI (bin/aps + lib/) to target
install_cli() {
  local target="$1"

  for f in "${CLI_FILES[@]}"; do
    download "$f" "$target/$f"
  done
  chmod +x "$target/bin/aps"
}

# Set up PATH so `aps` works without ./bin/ prefix
setup_path() {
  local target="$1"

  echo ""
  if command -v direnv &>/dev/null; then
    local envrc="$target/.envrc"
    if [[ -f "$envrc" ]] && grep -q 'PATH_add bin' "$envrc" 2>/dev/null; then
      info "PATH already configured in .envrc"
    elif ask_yn "Set up direnv so you can run 'aps' without ./bin/ prefix?" "y"; then
      if [[ -f "$envrc" ]]; then
        echo 'PATH_add bin' >> "$envrc"
      else
        echo 'PATH_add bin' > "$envrc"
      fi
      info "Added 'PATH_add bin' to .envrc"
      echo "  Run 'direnv allow' to activate"
    else
      info "To run aps without the path prefix, add to your .envrc:"
      echo "  PATH_add bin"
    fi
  else
    info "To run 'aps' without ./bin/ prefix, either:"
    echo "  - Install direnv and add 'PATH_add bin' to .envrc"
    echo "  - Or add 'export PATH=\"./bin:\$PATH\"' to your shell config"
  fi
}

# Two-step hook prompt
prompt_hooks() {
  local target="$1"

  echo ""
  if ask_yn "Install APS hooks into .claude/settings.local.json?" "y"; then
    (cd "$target" && bash aps-planning/scripts/install-hooks.sh)
  else
    if ask_yn "Would you like me to copy them for you to install/review later?" "y"; then
      info "Hook scripts are at: aps-planning/scripts/"
      echo "  Run ./aps-planning/scripts/install-hooks.sh when ready"
      echo "  See aps-planning/hooks.md for what each hook does"
    else
      info "Skipping hooks. You can install them later:"
      echo "  ./aps-planning/scripts/install-hooks.sh"
    fi
  fi
}

# --- Subcommands ---

cmd_init() {
  local target="."

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_init_help; exit 0 ;;
      *) target="$1"; shift ;;
    esac
  done

  local plans_dir="$target/plans"

  if [[ -d "$plans_dir" ]]; then
    error "plans/ directory already exists at $target"
    echo ""
    echo "To update an existing project:"
    echo "  aps update"
    echo ""
    echo "To reinstall from scratch:"
    echo "  rm -rf $plans_dir && aps init"
    exit 1
  fi

  echo ""
  info "Initialising APS in $target"
  echo ""

  # CLI (bin/aps + lib/)
  install_cli "$target"
  info "bin/aps + lib/ (CLI)"

  # Templates and rules
  install_plans "$target"
  install_index "$target"
  info "plans/ (templates, rules, index)"

  # Skill
  install_skill "$target"
  info "aps-planning/ (skill, reference, examples, hooks, scripts)"

  # Commands
  install_commands "$target"
  info ".claude/commands/ (plan, plan-status)"

  # Project APS policy config (bash scaffold path)
  # NOTE: PowerShell scaffold parity is tracked separately.
  write_apsrc_config "$target"

  echo ""
  echo "  bin/"
  echo "  └── aps                              <- CLI (lint, init, update)"
  echo ""
  echo "  plans/"
  echo "  ├── aps-rules.md                     <- Agent guidance (READ THIS)"
  echo "  ├── index.aps.md                     <- Your main plan (edit this)"
  echo "  ├── modules/"
  echo "  │   ├── .module.template.md          <- Template for modules"
  echo "  │   ├── .simple.template.md          <- Template for small features"
  echo "  │   └── .index-monorepo.template.md  <- Index for monorepos"
  echo "  ├── execution/"
  echo "  │   └── .steps.template.md           <- Template for steps"
  echo "  └── decisions/"
  echo ""
  echo "  aps-planning/"
  echo "  ├── SKILL.md                         <- Planning skill (core rules)"
  echo "  ├── reference.md                     <- APS format reference"
  echo "  ├── examples.md                      <- Real-world examples"
  echo "  ├── hooks.md                         <- Hook configuration guide"
  echo "  └── scripts/                         <- Hook install + session scripts"
  echo ""
  echo "  .claude/commands/"
  echo "  ├── plan.md                          <- /plan command"
  echo "  └── plan-status.md                   <- /plan-status command"
  echo ""
  echo "  .apsrc.yaml                          <- Project APS policy (versioned)"

  # Hooks
  prompt_hooks "$target"

  # PATH setup
  setup_path "$target"

  echo ""
  info "Next steps:"
  echo "  1. Edit plans/index.aps.md to define your plan"
  echo "  2. Copy templates to create modules (remove leading dot)"
  echo "  3. Use /plan in Claude Code to start planning"
  echo ""
}

cmd_init_help() {
  cat <<EOF
aps init - Create APS structure in a new project

Usage:
  aps init [target-dir]

Creates bin/aps CLI, plans/, aps-planning/ skill, .claude/commands/,
project policy file (.apsrc.yaml), and optionally installs hooks + PATH via direnv.

Refuses to run if plans/ already exists.

Options:
  --help    Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  aps init              # Init in current directory
  aps init ./my-project # Init in a subdirectory
EOF
}

cmd_update() {
  local target="."

  while [[ $# -gt 0 ]]; do
    case $1 in
      --help|-h) cmd_update_help; exit 0 ;;
      *) target="$1"; shift ;;
    esac
  done

  local plans_dir="$target/plans"

  if [[ ! -d "$plans_dir" ]]; then
    error "No plans/ directory found at $target"
    echo ""
    echo "To create a new APS project:"
    echo "  aps init"
    exit 1
  fi

  echo ""
  info "Updating APS in $target"
  echo ""

  # CLI (always update — this is how users get new features)
  install_cli "$target"
  info "bin/aps + lib/ (CLI)"

  # Templates and rules (preserves user specs)
  install_plans "$target"
  info "plans/ (templates, rules)"

  # Skill
  install_skill "$target"
  info "aps-planning/ (skill, reference, examples, hooks, scripts)"

  # Commands
  install_commands "$target"
  info ".claude/commands/ (plan, plan-status)"

  # Config (optional on update for existing projects)
  if [[ ! -f "$target/.apsrc.yaml" ]]; then
    # Avoid side effects in non-interactive contexts (CI/automation)
    local apsrc_default="y"
    if [[ ! -t 0 ]]; then
      apsrc_default="n"
    fi

    if ask_yn "No .apsrc.yaml found. Generate project APS policy now?" "$apsrc_default"; then
      write_apsrc_config "$target"
    else
      info "Skipped .apsrc.yaml generation (you can add it later)."
    fi
  fi

  # Hooks: prompt only if not already configured
  if ! has_aps_hooks "$target"; then
    prompt_hooks "$target"
  else
    echo ""
    info "Hooks already configured (not modified)."
    echo "  To update: ./aps-planning/scripts/install-hooks.sh"
  fi

  echo ""
  info "Your specs (index.aps.md, modules/*.aps.md) were NOT modified."
  echo ""
}

cmd_update_help() {
  cat <<EOF
aps update - Update APS templates, skill, CLI, and commands

Usage:
  aps update [target-dir]

Updates the CLI, templates, rules, skill files, and commands without
touching your specs (index.aps.md, modules/*.aps.md, execution/*.actions.md).

If hooks are not yet configured, prompts to install them.

Options:
  --help    Show this help

Environment:
  APS_VERSION   Git ref to download from (default: main)

Examples:
  aps update              # Update current directory
  aps update ./my-project # Update a subdirectory
EOF
}
