# gx installer for Windows
# Usage: irm https://raw.githubusercontent.com/joshuaboys/gx/main/install.ps1 | iex

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Repo = 'joshuaboys/gx'
$Asset = 'gx-windows-x64.exe'
$InstallDir = if ($env:LOCALAPPDATA) {
    Join-Path $env:LOCALAPPDATA 'gx\bin'
} else {
    Join-Path $HOME '.local\bin'
}
$GxBin = Join-Path $InstallDir 'gx.exe'

function Write-Info([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Blue
}

function Write-Warn([string]$Message) {
    Write-Host "warning: $Message" -ForegroundColor Yellow
}

function Get-ExpectedHash([string]$AssetName, [string]$SumsPath) {
    foreach ($line in Get-Content -LiteralPath $SumsPath) {
        if ($line -match '^([0-9a-fA-F]{64})\s+\*?(\S+)$') {
            if ($Matches[2] -eq $AssetName) {
                return $Matches[1].ToLowerInvariant()
            }
        }
    }
    return $null
}

function Install-GxPrebuilt {
    $base = "https://github.com/$Repo/releases/latest/download"
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("gx-install-" + [guid]::NewGuid().ToString('n'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        $bin = Join-Path $tmp $Asset
        $sums = Join-Path $tmp 'SHA256SUMS'
        Write-Info "Downloading prebuilt binary (windows-x64)..."
        try {
            Invoke-WebRequest -Uri "$base/$Asset" -OutFile $bin -UseBasicParsing
        } catch {
            throw @"
No prebuilt binary for windows-x64 in the latest release.
Supported targets: linux-x64, linux-aarch64, darwin-x64, darwin-aarch64, windows-x64.
Download manually from https://github.com/$Repo/releases, or build from
source with a Rust toolchain: 'cargo build --release -p gx'.
"@
        }
        Invoke-WebRequest -Uri "$base/SHA256SUMS" -OutFile $sums -UseBasicParsing
        $expected = Get-ExpectedHash -AssetName $Asset -SumsPath $sums
        if (-not $expected) {
            throw "Checksum for $Asset not found in SHA256SUMS"
        }
        $actual = (Get-FileHash -LiteralPath $bin -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            throw "Checksum verification failed for $Asset"
        }
        Write-Info 'Verified SHA-256 checksum'
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        Copy-Item -LiteralPath $bin -Destination $GxBin -Force
    } finally {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Install-GxShellIntegration {
    $profilePath = $PROFILE
    if (-not $profilePath) {
        Write-Warn 'Could not resolve $PROFILE. Add shell integration manually:'
        Write-Host '  Invoke-Expression (& gx shell-init powershell | Out-String)'
        return
    }
    $marker = '# gx'
    $initLine = 'Invoke-Expression (& gx shell-init powershell | Out-String)'
    if ((Test-Path -LiteralPath $profilePath) -and (Select-String -LiteralPath $profilePath -SimpleMatch $marker -Quiet)) {
        Write-Info "Shell integration already in $profilePath"
        return
    }
    $parent = Split-Path -Parent $profilePath
    if ($parent -and -not (Test-Path -LiteralPath $parent)) {
        New-Item -ItemType Directory -Path $parent -Force | Out-Null
    }
    Add-Content -LiteralPath $profilePath -Value "`r`n$marker`r`n$initLine"
    Write-Info "Added shell integration to $profilePath"
}

function Add-GxUserPath {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if ($null -eq $userPath) { $userPath = '' }
    $parts = $userPath -split ';' | Where-Object { $_ -ne '' }
    if ($parts -contains $InstallDir) {
        return
    }
    $newPath = if ($userPath.Trim() -eq '') { $InstallDir } else { "$userPath;$InstallDir" }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$InstallDir;$env:Path"
    Write-Info "Added $InstallDir to user PATH"
}

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
} catch {
    # Older hosts may not expose this setter; Invoke-WebRequest still tries TLS.
}

Write-Info 'Installing gx...'
Install-GxPrebuilt
Install-GxShellIntegration
Add-GxUserPath

Write-Host ''
Write-Info "gx installed to $GxBin"
Write-Info 'Restart PowerShell to pick up PATH and shell integration'
Write-Host ''
