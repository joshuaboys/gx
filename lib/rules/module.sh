#!/usr/bin/env bash
#
# Validation rules for module and simple templates
#

# E001: Missing ## Purpose section
check_e001_purpose() {
  local file="$1"
  if ! has_section "$file" "## Purpose"; then
    add_result "$file" "error" "E001" "Missing ## Purpose section"
    return 1
  fi
  return 0
}

# E002: Missing ## Work Items section
check_e002_work_items() {
  local file="$1"
  if ! has_section "$file" "## Work Items"; then
    add_result "$file" "error" "E002" "Missing ## Work Items section"
    return 1
  fi
  return 0
}

# E003: Missing ID/Status metadata table
check_e003_metadata() {
  local file="$1"
  if ! has_metadata_table "$file"; then
    add_result "$file" "error" "E003" "Missing ID/Status metadata table"
    return 1
  fi
  return 0
}

# W004: Empty section check (for module-specific sections)
check_w004_empty_sections_module() {
  local file="$1"
  local sections=("## Purpose" "## In Scope")

  for section in "${sections[@]}"; do
    if has_section "$file" "$section" && ! section_has_content "$file" "$section"; then
      local line
      line=$(get_line_number "$file" "^${section}$")
      add_result "$file" "warning" "W004" "Empty section: $section" "$line"
    fi
  done
}

# W005: Status=Ready but no work items
check_w005_ready_no_items() {
  local file="$1"
  local status
  status=$(get_status "$file")

  if [[ "$status" == "Ready" ]]; then
    local items
    items=$(get_work_items "$file")
    if [[ -z "$items" ]]; then
      add_result "$file" "warning" "W005" "Status is Ready but no work items defined"
    fi
  fi
}

# Run all module/simple rules
lint_module() {
  local file="$1"
  local has_errors=false

  check_e001_purpose "$file" || has_errors=true
  check_e002_work_items "$file" || has_errors=true
  check_e003_metadata "$file" || has_errors=true

  check_w004_empty_sections_module "$file"
  check_w005_ready_no_items "$file"

  # Check work items if the section exists
  if has_section "$file" "## Work Items"; then
    lint_work_items "$file" || has_errors=true
  fi

  $has_errors && return 1
  return 0
}
