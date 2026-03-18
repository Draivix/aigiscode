# AigisCode Installer for Windows
# Usage: irm https://raw.githubusercontent.com/Draivix/aigiscode/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$VERSION = if ($env:AIGISCODE_VERSION) { $env:AIGISCODE_VERSION } else { "0.1.0" }
$REPO = "Draivix/aigiscode"
$BIN_NAME = "aigiscode"

function Write-Info { param($msg) Write-Host "  > " -ForegroundColor Blue -NoNewline; Write-Host $msg }
function Write-Ok { param($msg) Write-Host "  ✓ " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn { param($msg) Write-Host "  ! " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err { param($msg) Write-Host "  ✗ " -ForegroundColor Red -NoNewline; Write-Host $msg }

function Show-Banner {
    Write-Host ""
    Write-Host "    ╔═══════════════════════════════════════╗" -ForegroundColor Magenta
    Write-Host "    ║     AigisCode Installer  v$VERSION       ║" -ForegroundColor Magenta
    Write-Host "    ║     Native Rust CLI                  ║" -ForegroundColor Magenta
    Write-Host "    ╚═══════════════════════════════════════╝" -ForegroundColor Magenta
    Write-Host ""
}

function Find-Cargo {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargo) {
        return $cargo.Source
    }
    $cargoPath = "$env:USERPROFILE\.cargo\bin\cargo.exe"
    if (Test-Path $cargoPath) {
        return $cargoPath
    }
    return $null
}

function Install-WithCargo {
    param($CargoCmd)

    Write-Info "Installing native Rust CLI with cargo..."
    & $CargoCmd install --git "https://github.com/$REPO.git" --locked aigiscore --bin $BIN_NAME 2>$null
    if ($LASTEXITCODE -ne 0) {
        & $CargoCmd install --git "https://github.com/$REPO.git" aigiscore --bin $BIN_NAME
        if ($LASTEXITCODE -ne 0) { throw "cargo install failed" }
    }
}

Show-Banner
Write-Info "Detected: Windows/$([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)"

$cargoCmd = Find-Cargo
if (-not $cargoCmd) {
    Write-Err "Cargo is required but was not found."
    Write-Host ""
    Write-Host "  Install Rust first:" -ForegroundColor Yellow
    Write-Host "    winget install Rustlang.Rustup" -ForegroundColor Green
    Write-Host "  Then restart your terminal and rerun this installer." -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

Write-Ok "Found $(& $cargoCmd --version 2>&1)"
Install-WithCargo -CargoCmd $cargoCmd

try {
    $ver = & $BIN_NAME --version 2>$null
    Write-Ok "AigisCode installed successfully! ($ver)"
} catch {
    Write-Ok "AigisCode installed. Restart your terminal to use it."
}

Write-Host ""
Write-Host "  Get started:" -ForegroundColor White
Write-Host "    aigiscode analyze .    " -ForegroundColor Green -NoNewline; Write-Host "# Analyze current directory and write native artifacts"
Write-Host "    aigiscode surface .    " -ForegroundColor Green -NoNewline; Write-Host "# Generate architecture-surface.json"
Write-Host "    aigiscode --help       " -ForegroundColor Green -NoNewline; Write-Host "# See all commands"
Write-Host ""
Write-Host "    Docs:    https://aigiscode.com/docs" -ForegroundColor Blue
Write-Host "    GitHub:  https://github.com/$REPO" -ForegroundColor Blue
Write-Host ""
