#
# APS CLI Index Validation Rules
# Port of lib/rules/index.sh
#

# Dependencies (Output, Common) must be imported by the entry point.

# E004: Missing ## Modules section
function Test-E004Modules {
    param([string]$File)
    if (-not (Test-ApsSection -FilePath $File -SectionHeader "## Modules")) {
        Add-ApsResult -Path $File -Type "error" -Code "E004" -Message "Missing ## Modules section"
        return $false
    }
    return $true
}

# W004: Empty section check (index-specific sections)
function Test-W004EmptySectionsIndex {
    param([string]$File)
    $sections = @("## Overview", "## Problem & Success Criteria", "## Modules")
    foreach ($section in $sections) {
        if ((Test-ApsSection -FilePath $File -SectionHeader $section) -and
            -not (Test-ApsSectionHasContent -FilePath $File -SectionHeader $section)) {
            $line = Get-ApsLineNumber -FilePath $File -Pattern "^$([regex]::Escape($section))$"
            Add-ApsResult -Path $File -Type "warning" -Code "W004" -Message "Empty section: $section" -Line "$line"
        }
    }
}

# Run all index rules
function Invoke-ApsIndexLint {
    param([string]$File)
    $hasErrors = $false

    if (-not (Test-E004Modules -File $File)) { $hasErrors = $true }
    Test-W004EmptySectionsIndex -File $File

    return (-not $hasErrors)
}

Export-ModuleMember -Function 'Invoke-ApsIndexLint'
