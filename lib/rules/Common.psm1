#
# APS CLI Common Validation Helpers
# Port of lib/rules/common.sh — section detection, metadata parsing
#

function Test-ApsSection {
    param([string]$FilePath, [string]$SectionHeader)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $false }
    foreach ($line in $lines) {
        if ($line -ceq $SectionHeader) { return $true }
    }
    return $false
}

function Test-ApsMetadataTable {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $false }
    $limit = [Math]::Min(20, $lines.Count)
    for ($i = 0; $i -lt $limit; $i++) {
        if ($lines[$i] -match '^\| *ID *\|') { return $true }
    }
    return $false
}

function Get-ApsSectionContent {
    param([string]$FilePath, [string]$SectionHeader)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $found = $false
    $content = [System.Collections.ArrayList]::new()
    foreach ($line in $lines) {
        if ($line -ceq $SectionHeader) {
            $found = $true
            continue
        }
        if ($found -and $line -match '^## ') { break }
        if ($found) { $null = $content.Add($line) }
    }
    return @($content)
}

function Test-ApsSectionHasContent {
    param([string]$FilePath, [string]$SectionHeader)
    $content = Get-ApsSectionContent -FilePath $FilePath -SectionHeader $SectionHeader
    foreach ($line in $content) {
        $trimmed = $line.Trim()
        if ($trimmed -eq "") { continue }
        if ($trimmed -match '^<!--.*-->$') { continue }
        if ($trimmed -match '^<!--') { continue }
        return $true
    }
    return $false
}

function Get-ApsWorkItems {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $results = [System.Collections.ArrayList]::new()
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^### [A-Za-z]+-[0-9]+:') {
            $null = $results.Add([PSCustomObject]@{
                LineNumber = $i + 1
                Header     = $lines[$i]
            })
        }
    }
    return @($results)
}

function Get-ApsModuleId {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return "" }
    $foundHeader = $false
    foreach ($line in $lines) {
        if ($line -match '^\| *ID *\|') {
            $foundHeader = $true
            continue
        }
        if ($foundHeader -and $line -match '^\|') {
            if ($line -match '^\| *-+') { continue }
            $cells = ($line -split '\|') | ForEach-Object { $_.Trim() } | Where-Object { $_ -ne '' }
            if ($cells.Count -gt 0) { return $cells[0] }
        }
    }
    return ""
}

function Get-ApsStatus {
    param([string]$FilePath)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return "" }
    $statusCol = -1
    $foundHeader = $false
    foreach ($line in $lines) {
        if ($line -match '^\| *ID *\|') {
            $cols = ($line -split '\|') | ForEach-Object { $_.Trim() }
            for ($i = 0; $i -lt $cols.Count; $i++) {
                if ($cols[$i] -ceq "Status") { $statusCol = $i }
            }
            $foundHeader = $true
            continue
        }
        if ($foundHeader -and $statusCol -ge 0 -and $line -match '^\|[^-]' -and $line -notmatch '^\| *ID *\|') {
            $vals = ($line -split '\|') | ForEach-Object { $_.Trim() }
            if ($statusCol -lt $vals.Count) { return $vals[$statusCol] }
        }
    }
    return ""
}

function Get-ApsLineNumber {
    param([string]$FilePath, [string]$Pattern)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return $null }
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match $Pattern) { return ($i + 1) }
    }
    return $null
}

function Get-ApsWorkItemContent {
    param([string]$FilePath, [int]$StartLine)
    $lines = Get-Content -LiteralPath $FilePath -ErrorAction SilentlyContinue
    if (-not $lines) { return @() }
    $content = [System.Collections.ArrayList]::new()
    for ($i = $StartLine; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^###? ') { break }
        $null = $content.Add($lines[$i])
    }
    return @($content)
}

Export-ModuleMember -Function @(
    'Test-ApsSection'
    'Test-ApsMetadataTable'
    'Get-ApsSectionContent'
    'Test-ApsSectionHasContent'
    'Get-ApsWorkItems'
    'Get-ApsModuleId'
    'Get-ApsStatus'
    'Get-ApsLineNumber'
    'Get-ApsWorkItemContent'
)
