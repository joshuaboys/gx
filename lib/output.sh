#!/usr/bin/env bash
#
# Output formatting for APS CLI
#

# Color support (disabled if not a terminal)
if [[ -t 1 ]]; then
  RED='\033[0;31m'
  YELLOW='\033[1;33m'
  GREEN='\033[0;32m'
  GRAY='\033[0;90m'
  BOLD='\033[1m'
  NC='\033[0m'
else
  RED=''
  YELLOW=''
  GREEN=''
  GRAY=''
  BOLD=''
  NC=''
fi

error() { echo -e "${RED}error:${NC} $1" >&2; }
warn() { echo -e "${YELLOW}warning:${NC} $1" >&2; }
info() { echo -e "${GREEN}aps:${NC} $1"; }

# Global results storage
declare -a FILE_RESULTS=()
TOTAL_FILES=0
TOTAL_ERRORS=0
TOTAL_WARNINGS=0

# Add a result for a file
# Usage: add_result "path" "type" "code" "message" "line"
add_result() {
  local path="$1"
  local type="$2"    # error or warning
  local code="$3"
  local message="$4"
  local line="${5:-}"

  FILE_RESULTS+=("$path|$type|$code|$message|$line")

  if [[ "$type" == "error" ]]; then
    ((TOTAL_ERRORS++)) || true
  elif [[ "$type" == "warning" ]]; then
    ((TOTAL_WARNINGS++)) || true
  fi
  # "ok" type doesn't increment counters
}

# Print results in text format
print_text_results() {
  local current_file=""
  local file_has_issues=false

  for result in "${FILE_RESULTS[@]}"; do
    IFS='|' read -r path type code message line <<< "$result"

    if [[ "$path" != "$current_file" ]]; then
      # Print previous file status
      if [[ -n "$current_file" ]]; then
        if [[ "$file_has_issues" == false ]]; then
          echo -e "  ${GREEN}✓${NC} valid"
        fi
        echo ""
      fi

      current_file="$path"
      file_has_issues=false
      echo -e "${BOLD}$path${NC}"
    fi

    if [[ "$code" != "OK" ]]; then
      file_has_issues=true

      local color="$RED"
      [[ "$type" == "warning" ]] && color="$YELLOW"

      local line_info=""
      [[ -n "$line" ]] && line_info=" ${GRAY}(line $line)${NC}"

      echo -e "  ${color}$code:${NC} $message$line_info"
    fi
  done

  # Print last file status
  if [[ -n "$current_file" && "$file_has_issues" == false ]]; then
    echo -e "  ${GREEN}✓${NC} valid"
  fi

  echo ""

  # Summary
  local summary="$TOTAL_FILES file"
  [[ $TOTAL_FILES -ne 1 ]] && summary+="s"
  summary+=" checked"

  if [[ $TOTAL_ERRORS -eq 0 && $TOTAL_WARNINGS -eq 0 ]]; then
    echo -e "${GREEN}$summary, no issues${NC}"
  else
    local parts=()
    if [[ $TOTAL_ERRORS -gt 0 ]]; then
      local err_s=""; [[ $TOTAL_ERRORS -ne 1 ]] && err_s="s"
      parts+=("${RED}$TOTAL_ERRORS error${err_s}${NC}")
    fi
    if [[ $TOTAL_WARNINGS -gt 0 ]]; then
      local warn_s=""; [[ $TOTAL_WARNINGS -ne 1 ]] && warn_s="s"
      parts+=("${YELLOW}$TOTAL_WARNINGS warning${warn_s}${NC}")
    fi
    local joined=""
    for i in "${!parts[@]}"; do
      [[ $i -gt 0 ]] && joined+=", "
      joined+="${parts[$i]}"
    done
    echo -e "$summary, $joined"
  fi
}

# Print results in JSON format
print_json_results() {
  local json='{"files":['
  local current_file=""
  local file_type=""
  local errors=""
  local warnings=""

  for result in "${FILE_RESULTS[@]}"; do
    IFS='|' read -r path type code message line <<< "$result"

    if [[ "$path" != "$current_file" ]]; then
      # Close previous file object
      if [[ -n "$current_file" ]]; then
        json+="\"errors\":[$errors],\"warnings\":[$warnings]},"
      fi

      current_file="$path"
      file_type="${FILE_TYPES[$path]:-unknown}"
      errors=""
      warnings=""

      json+="{\"path\":\"$path\",\"type\":\"$file_type\","
    fi

    if [[ "$code" != "OK" ]]; then
      local entry="{\"code\":\"$code\",\"message\":\"$message\""
      [[ -n "$line" ]] && entry+=",\"line\":$line"
      entry+="}"

      if [[ "$type" == "error" ]]; then
        [[ -n "$errors" ]] && errors+=","
        errors+="$entry"
      else
        [[ -n "$warnings" ]] && warnings+=","
        warnings+="$entry"
      fi
    fi
  done

  # Close last file object
  if [[ -n "$current_file" ]]; then
    json+="\"errors\":[$errors],\"warnings\":[$warnings]}"
  fi

  json+="],\"summary\":{\"files\":$TOTAL_FILES,\"errors\":$TOTAL_ERRORS,\"warnings\":$TOTAL_WARNINGS}}"

  echo "$json"
}
