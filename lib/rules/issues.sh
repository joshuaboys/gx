#!/usr/bin/env bash
#
# Validation rules for issues.md tracker
#

# E010: Missing ## Issues section
check_e010_issues_section() {
  local file="$1"
  if ! has_section "$file" "## Issues"; then
    add_result "$file" "error" "E010" "Missing ## Issues section"
    return 1
  fi
  return 0
}

# E011: Missing ## Questions section
check_e011_questions_section() {
  local file="$1"
  if ! has_section "$file" "## Questions"; then
    add_result "$file" "error" "E011" "Missing ## Questions section"
    return 1
  fi
  return 0
}

# W010: Issue missing required fields (Status, Discovered, Severity)
check_w010_issue_fields() {
  local file="$1"
  local issues_content
  issues_content=$(get_section_content "$file" "## Issues")

  # Find all issue headers (### ISS-NNN: ...)
  local issue_headers
  issue_headers=$(echo "$issues_content" | grep -nE '^### ISS-[0-9]{3}:' 2>/dev/null || true)

  while IFS= read -r header_line; do
    [[ -z "$header_line" ]] && continue
    local line_num
    line_num=$(echo "$header_line" | cut -d: -f1)
    local issue_id
    issue_id=$(echo "$header_line" | grep -oE 'ISS-[0-9]{3}' | head -1)

    # Get content until next ### or ## heading
    local issue_content
    issue_content=$(echo "$issues_content" | awk -v start="$line_num" '
      NR >= start && NR == start { found=1; next }
      found && /^###? / { exit }
      found { print }
    ')

    # Check for Status field in table
    if ! echo "$issue_content" | grep -qE '^\| *Status *\|'; then
      add_result "$file" "warning" "W010" "$issue_id: Missing Status field in metadata table" "$line_num"
    fi

    # Check for Discovered field
    if ! echo "$issue_content" | grep -qE '^\| *Discovered *\|'; then
      add_result "$file" "warning" "W010" "$issue_id: Missing Discovered field (traceability)" "$line_num"
    fi

    # Check for Severity field
    if ! echo "$issue_content" | grep -qE '^\| *Severity *\|'; then
      add_result "$file" "warning" "W010" "$issue_id: Missing Severity field in metadata table" "$line_num"
    fi
  done <<< "$issue_headers"
}

# W011: Question missing required fields (Status, Discovered, Priority)
check_w011_question_fields() {
  local file="$1"
  local questions_content
  questions_content=$(get_section_content "$file" "## Questions")

  # Find all question headers (### Q-NNN: ...)
  local question_headers
  question_headers=$(echo "$questions_content" | grep -nE '^### Q-[0-9]{3}:' 2>/dev/null || true)

  while IFS= read -r header_line; do
    [[ -z "$header_line" ]] && continue
    local line_num
    line_num=$(echo "$header_line" | cut -d: -f1)
    local question_id
    question_id=$(echo "$header_line" | grep -oE 'Q-[0-9]{3}' | head -1)

    # Get content until next ### or ## heading
    local question_content
    question_content=$(echo "$questions_content" | awk -v start="$line_num" '
      NR >= start && NR == start { found=1; next }
      found && /^###? / { exit }
      found { print }
    ')

    # Check for Status field in table
    if ! echo "$question_content" | grep -qE '^\| *Status *\|'; then
      add_result "$file" "warning" "W011" "$question_id: Missing Status field in metadata table" "$line_num"
    fi

    # Check for Discovered field
    if ! echo "$question_content" | grep -qE '^\| *Discovered *\|'; then
      add_result "$file" "warning" "W011" "$question_id: Missing Discovered field (traceability)" "$line_num"
    fi

    # Check for Priority field
    if ! echo "$question_content" | grep -qE '^\| *Priority *\|'; then
      add_result "$file" "warning" "W011" "$question_id: Missing Priority field in metadata table" "$line_num"
    fi
  done <<< "$question_headers"
}

# W012: Issue ID format warning (also catches wrong-case prefixes)
check_w012_issue_id_format() {
  local file="$1"

  # Find any ### ISS- header that doesn't match ISS-NNN: format (exactly 3 digits)
  local bad_ids
  bad_ids=$(grep -nE '^### ISS-' "$file" 2>/dev/null | grep -vE '^[0-9]+:### ISS-[0-9]{3}:' || true)

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    local line_num
    line_num=$(echo "$line" | cut -d: -f1)
    add_result "$file" "warning" "W012" "Issue ID should be ISS-NNN format (e.g., ISS-001)" "$line_num"
  done <<< "$bad_ids"

  # Catch wrong-case prefixes (e.g., iss-, Iss-)
  local wrong_case
  wrong_case=$(grep -nEi '^### iss-' "$file" 2>/dev/null | grep -vE '^[0-9]+:### ISS-' || true)

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    local line_num
    line_num=$(echo "$line" | cut -d: -f1)
    add_result "$file" "warning" "W012" "Issue ID prefix must be uppercase ISS- (found wrong casing)" "$line_num"
  done <<< "$wrong_case"
}

# W013: Question ID format warning (also catches wrong-case prefixes)
check_w013_question_id_format() {
  local file="$1"

  # Find any ### Q- header that doesn't match Q-NNN: format (exactly 3 digits)
  local bad_ids
  bad_ids=$(grep -nE '^### Q-' "$file" 2>/dev/null | grep -vE '^[0-9]+:### Q-[0-9]{3}:' || true)

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    local line_num
    line_num=$(echo "$line" | cut -d: -f1)
    add_result "$file" "warning" "W013" "Question ID should be Q-NNN format (e.g., Q-001)" "$line_num"
  done <<< "$bad_ids"

  # Catch wrong-case prefixes (e.g., q-)
  local wrong_case
  wrong_case=$(grep -nEi '^### q-' "$file" 2>/dev/null | grep -vE '^[0-9]+:### Q-' || true)

  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    local line_num
    line_num=$(echo "$line" | cut -d: -f1)
    add_result "$file" "warning" "W013" "Question ID prefix must be uppercase Q- (found wrong casing)" "$line_num"
  done <<< "$wrong_case"
}

# Run all issues rules
lint_issues() {
  local file="$1"
  local has_errors=false

  check_e010_issues_section "$file" || has_errors=true
  check_e011_questions_section "$file" || has_errors=true

  # Only run field checks if sections exist
  if has_section "$file" "## Issues"; then
    check_w010_issue_fields "$file"
    check_w012_issue_id_format "$file"
  fi

  if has_section "$file" "## Questions"; then
    check_w011_question_fields "$file"
    check_w013_question_id_format "$file"
  fi

  $has_errors && return 1
  return 0
}
