#!/usr/bin/env bash
#
# Validation rules for index template
#

# E004: Missing ## Modules section
check_e004_modules() {
  local file="$1"
  if ! has_section "$file" "## Modules"; then
    add_result "$file" "error" "E004" "Missing ## Modules section"
    return 1
  fi
  return 0
}

# W004: Empty section check (for index-specific sections)
check_w004_empty_sections_index() {
  local file="$1"
  local sections=("## Overview" "## Problem & Success Criteria" "## Modules")

  for section in "${sections[@]}"; do
    if has_section "$file" "$section" && ! section_has_content "$file" "$section"; then
      local line
      line=$(get_line_number "$file" "^${section}$")
      add_result "$file" "warning" "W004" "Empty section: $section" "$line"
    fi
  done
}

# Run all index rules
lint_index() {
  local file="$1"
  local has_errors=false

  check_e004_modules "$file" || has_errors=true
  check_w004_empty_sections_index "$file"

  $has_errors && return 1
  return 0
}
