#
# APS CLI Issues Tracker Validation Rules
# Port of lib/rules/issues.sh
#

# Dependencies (Output, Common) must be imported by the entry point.

# E010: Missing ## Issues section
function Test-E010IssuesSection {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Issues")) {
        Add-ApsResult -Path $File -Type "error" -Code "E010" -Message "Missing ## Issues section"
        return $false
    }
    return $true
}

# E011: Missing ## Questions section
function Test-E011QuestionsSection {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Questions")) {
        Add-ApsResult -Path $File -Type "error" -Code "E011" -Message "Missing ## Questions section"
        return $false
    }
    return $true
}

# W010: Issue missing required fields (Status, Discovered, Severity)
function Test-W010IssueFields {
    param([string]$File)
    $sectionContent = Get-ApsSectionContent -FilePath $File -SectionHeader "## Issues"
    if ($sectionContent.Count -eq 0) { return }

    for ($i = 0; $i -lt $sectionContent.Count; $i++) {
        if ($sectionContent[$i] -cmatch '^### ISS-[0-9]{3}:') {
            $issueId = if ($sectionContent[$i] -cmatch '(ISS-[0-9]{3})') { $Matches[1] } else { "ISS-???" }
            $lineNum = $i + 1  # relative to section

            # Collect content until next ### or ## heading
            $issueContent = @()
            for ($j = $i + 1; $j -lt $sectionContent.Count; $j++) {
                if ($sectionContent[$j] -cmatch '^###? ') { break }
                $issueContent += $sectionContent[$j]
            }
            $text = $issueContent -join "`n"

            if ($text -cnotmatch '(?m)^\| *Status *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W010" -Message "$issueId`: Missing Status field in metadata table" -Line "$lineNum"
            }
            if ($text -cnotmatch '(?m)^\| *Discovered *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W010" -Message "$issueId`: Missing Discovered field (traceability)" -Line "$lineNum"
            }
            if ($text -cnotmatch '(?m)^\| *Severity *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W010" -Message "$issueId`: Missing Severity field in metadata table" -Line "$lineNum"
            }
        }
    }
}

# W011: Question missing required fields (Status, Discovered, Priority)
function Test-W011QuestionFields {
    param([string]$File)
    $sectionContent = Get-ApsSectionContent -FilePath $File -SectionHeader "## Questions"
    if ($sectionContent.Count -eq 0) { return }

    for ($i = 0; $i -lt $sectionContent.Count; $i++) {
        if ($sectionContent[$i] -cmatch '^### Q-[0-9]{3}:') {
            $questionId = if ($sectionContent[$i] -cmatch '(Q-[0-9]{3})') { $Matches[1] } else { "Q-???" }
            $lineNum = $i + 1

            $questionContent = @()
            for ($j = $i + 1; $j -lt $sectionContent.Count; $j++) {
                if ($sectionContent[$j] -cmatch '^###? ') { break }
                $questionContent += $sectionContent[$j]
            }
            $text = $questionContent -join "`n"

            if ($text -cnotmatch '(?m)^\| *Status *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W011" -Message "$questionId`: Missing Status field in metadata table" -Line "$lineNum"
            }
            if ($text -cnotmatch '(?m)^\| *Discovered *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W011" -Message "$questionId`: Missing Discovered field (traceability)" -Line "$lineNum"
            }
            if ($text -cnotmatch '(?m)^\| *Priority *\|') {
                Add-ApsResult -Path $File -Type "warning" -Code "W011" -Message "$questionId`: Missing Priority field in metadata table" -Line "$lineNum"
            }
        }
    }
}

# W012: Issue ID format warning (also catches wrong-case prefixes)
function Test-W012IssueIdFormat {
    param([string]$File)
    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    if (-not $lines) { return }

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $lineNum = $i + 1

        # Correct prefix but wrong digit format
        if ($line -cmatch '^### ISS-' -and $line -cnotmatch '^### ISS-[0-9]{3}:') {
            Add-ApsResult -Path $File -Type "warning" -Code "W012" `
                -Message "Issue ID should be ISS-NNN format (e.g., ISS-001)" -Line "$lineNum"
        }

        # Wrong-case prefix (case-insensitive match but not uppercase ISS-)
        if ($line -imatch '^### iss-' -and $line -cnotmatch '^### ISS-') {
            Add-ApsResult -Path $File -Type "warning" -Code "W012" `
                -Message "Issue ID prefix must be uppercase ISS- (found wrong casing)" -Line "$lineNum"
        }
    }
}

# W013: Question ID format warning (also catches wrong-case prefixes)
function Test-W013QuestionIdFormat {
    param([string]$File)
    $lines = Get-Content -LiteralPath $File -ErrorAction SilentlyContinue
    if (-not $lines) { return }

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]
        $lineNum = $i + 1

        if ($line -cmatch '^### Q-' -and $line -cnotmatch '^### Q-[0-9]{3}:') {
            Add-ApsResult -Path $File -Type "warning" -Code "W013" `
                -Message "Question ID should be Q-NNN format (e.g., Q-001)" -Line "$lineNum"
        }

        if ($line -imatch '^### q-' -and $line -cnotmatch '^### Q-') {
            Add-ApsResult -Path $File -Type "warning" -Code "W013" `
                -Message "Question ID prefix must be uppercase Q- (found wrong casing)" -Line "$lineNum"
        }
    }
}

# Run all issues rules
function Invoke-ApsIssuesLint {
    param([string]$File)
    $hasErrors = $false

    if (-not (Test-E010IssuesSection -File $File)) { $hasErrors = $true }
    if (-not (Test-E011QuestionsSection -File $File)) { $hasErrors = $true }

    if (Test-ApsSection -FilePath $File -SectionHeader "## Issues") {
        Test-W010IssueFields -File $File
        Test-W012IssueIdFormat -File $File
    }
    if (Test-ApsSection -FilePath $File -SectionHeader "## Questions") {
        Test-W011QuestionFields -File $File
        Test-W013QuestionIdFormat -File $File
    }

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsIssuesLint'
