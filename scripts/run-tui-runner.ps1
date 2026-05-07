# TEAM_006: Script to run the TUI Module Runner with enhanced error handling
# This script launches all Wireframe AI modules in a single TUI window
param(
    [switch]$Verbose
)

# Load common utilities
$commonPath = Join-Path $PSScriptRoot "common.ps1"
if (Test-Path $commonPath) {
    . $commonPath
} else {
    Write-Error "common.ps1 not found at $commonPath"
    exit 1
}

$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path "$PSScriptRoot/.."

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - TUI Module Runner            |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""
Write-Host "  Keyboard shortcuts:" -ForegroundColor $global:ScriptColors.Info
Write-Muted "    q  - Quit"
Write-Muted "    Ctrl+C - Stop all and quit"
Write-Muted "    Up/Down - Select module"
Write-Muted "    Enter - Toggle selected module"
Write-Muted "    s - Start all modules"
Write-Muted "    x - Stop all modules"
Write-Host ""

# Check if cargo is available
if (-not (Test-Command "cargo")) {
    Write-Error "Rust toolchain (cargo) not found. Install from https://rustup.rs"
    exit 1
}
Write-Ok "Rust toolchain found"

Push-Location $RootDir
try {
    Write-Step "Starting TUI Module Runner..."
    if ($Verbose) {
        cargo run --release -p wireframe-tui-runner
    } else {
        cargo run --release -p wireframe-tui-runner 2>&1
    }
} catch {
    Write-Error "Failed to start TUI Module Runner: $($_.Exception.Message)"
    if ($Verbose) {
        Write-Host $_.ScriptStackTrace
    }
    exit 1
} finally {
    Pop-Location
}
