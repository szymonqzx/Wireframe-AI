#!/usr/bin/env pwsh
# TEAM_006: Build all modules in release mode with enhanced error handling and progress
param(
    [switch]$Verbose,
    [switch]$SkipCache
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

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - Release Build               |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

$targets = @(
    "wireframe-ai-interface",
    "wireframe-ai-context-core",
    "wireframe-ai-orchestrator-core",
    "wireframe-ai-sandbox-core"
)

$cacheDir = Join-Path $PSScriptRoot ".build-cache"
$failedTargets = @()

Write-Step "Starting release build..."
Write-Muted "Targets: $($targets -join ', ')"

# Check if cargo is available
if (-not (Test-Command "cargo")) {
    Write-Error "Rust toolchain (cargo) not found. Install from https://rustup.rs"
    exit 1
}
Write-Ok "Rust toolchain found"

# Validate workspace structure
$workspaceRoot = Resolve-Path "$PSScriptRoot/.."
if (-not (Test-Path "$workspaceRoot/Cargo.toml")) {
    Write-Error "Cargo.toml not found in workspace root: $workspaceRoot"
    exit 1
}
Write-Ok "Workspace structure validated"

# Create cache directory
if (-not (Test-Path $cacheDir)) {
    New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
}

$buildTimer = Start-Timer

foreach ($t in $targets) {
    $cacheFile = Join-Path $cacheDir "$t.hash"
    $sourceHash = $null

    # Check cache if not skipped
    if (-not $SkipCache -and (Test-Path $cacheFile)) {
        $cachedHash = Get-Content $cacheFile -Raw
        # Get actual file hash for the package
        $packagePath = Join-Path $workspaceRoot $t
        if (Test-Path $packagePath) {
            $sourceFiles = Get-ChildItem -Path $packagePath -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue
            if ($sourceFiles) {
                $sourceHash = ($sourceFiles | ForEach-Object { $_.LastWriteTime.Ticks } | Measure-Object -Sum).Sum
            }
        }

        if ($cachedHash -eq $sourceHash -and $sourceHash) {
            Write-Muted "  $t (cached, skipping)"
            continue
        }
    }

    Write-Host "  Building $t..." -NoNewline

    try {
        $result = cargo build --release -p $t 2>&1
        $success = $LASTEXITCODE -eq 0

        if ($Verbose) {
            Write-Host ""
            Write-Host $result
        }

        if ($success) {
            Write-Host " OK" -ForegroundColor $global:ScriptColors.Ok

            # Update cache
            if (-not $SkipCache) {
                $packagePath = Join-Path $workspaceRoot $t
                if (Test-Path $packagePath) {
                    $sourceFiles = Get-ChildItem -Path $packagePath -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue
                    if ($sourceFiles) {
                        $sourceHash = ($sourceFiles | ForEach-Object { $_.LastWriteTime.Ticks } | Measure-Object -Sum).Sum
                        $sourceHash | Out-File $cacheFile
                    }
                }
            }
        } else {
            Write-Host " FAILED" -ForegroundColor $global:ScriptColors.Err
            if (-not $Verbose) {
                Write-Host $result
            }
            $failedTargets += $t
        }
    } catch {
        Write-Host " ERROR" -ForegroundColor $global:ScriptColors.Err
        Write-Error "Build failed for $t: $($_.Exception.Message)"
        $failedTargets += $t
    }
}

$buildDuration = Stop-Timer -Timer $buildTimer
$durationStr = Format-Duration -Duration $buildDuration

Write-Host ""
Write-Step "Build summary"
Write-Host ""

if ($failedTargets.Count -eq 0) {
    Write-Ok "All targets built successfully"
    Write-Muted "Duration: $durationStr"
    Write-Host ""
    Write-Host "  Binaries in target/release/" -ForegroundColor $global:ScriptColors.Info
    exit 0
} else {
    Write-Error "Failed targets: $($failedTargets -join ', ')"
    Write-Muted "Duration: $durationStr"
    exit 1
}
