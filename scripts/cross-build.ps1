#!/usr/bin/env pwsh
# TEAM_006: Cross-compile Wireframe AI for all supported platforms with parallel builds
# Prerequisites:
#   cargo install cross
#   or use native targets (see .cargo/config.toml)
param(
    [switch]$Parallel,
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

$targets = @(
    @{ name = "Linux x86_64";    target = "x86_64-unknown-linux-gnu" },
    @{ name = "Linux ARM64";     target = "aarch64-unknown-linux-gnu" },
    @{ name = "Windows x86_64";  target = "x86_64-pc-windows-gnu" },
    @{ name = "macOS ARM64";     target = "aarch64-apple-darwin" }
)

$packages = @(
    "wireframe-ai-interface",
    "wireframe-ai-context-core",
    "wireframe-ai-orchestrator-core",
    "wireframe-ai-sandbox-core"
)

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - Cross-Platform Build         |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

Write-Step "Starting cross-platform build..."
Write-Muted "Platforms: $($targets.Count)"
Write-Muted "Packages: $($packages.Count)"

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

# Check for cross if parallel mode is requested
if ($Parallel -and -not (Test-Command "cross")) {
    Write-Warn "cross tool not found. Parallel build requires 'cargo install cross'"
    Write-InfoMsg "Falling back to sequential build"
    $Parallel = $false
}

$buildTimer = Start-Timer
$failedBuilds = @()
$totalBuilds = $targets.Count * $packages.Count
$completedBuilds = 0

if ($Parallel) {
    Write-InfoMsg "Parallel build mode enabled"

    # Build all packages for all targets in parallel
    $jobs = @()

    foreach ($pkg in $packages) {
        foreach ($plat in $targets) {
            $job = Start-Job -ScriptBlock {
                param($pkg, $plat, $verbose)
                $ErrorActionPreference = "Stop"

                if ($verbose) {
                    cargo build --release --target $plat.target -p $pkg 2>&1
                } else {
                    cargo build --release --target $plat.target -p $pkg 2>&1 | Out-Null
                }

                return @{
                    Package = $pkg
                    Platform = $plat.name
                    Success = $LASTEXITCODE -eq 0
                }
            } -ArgumentList $pkg, $plat, $Verbose

            $jobs += $job
        }
    }

    # Wait for all jobs and collect results
    foreach ($job in $jobs) {
        $result = Receive-Job -Job $job -Wait
        Remove-Job -Job $job

        $completedBuilds++
        $progress = [math]::Floor(($completedBuilds / $totalBuilds) * 100)
        Write-Progress -Activity "Building $($result.Package) for $($result.Platform)" -PercentComplete $progress

        if ($result.Success) {
            Write-Ok "$($result.Package) for $($result.Platform)"
        } else {
            Write-Error "$($result.Package) for $($result.Platform)"
            $failedBuilds += "$($result.Package)@$($result.Platform)"
        }
    }

    Clear-Progress
} else {
    Write-InfoMsg "Sequential build mode"

    foreach ($pkg in $packages) {
        foreach ($plat in $targets) {
            $completedBuilds++
            $progress = [math]::Floor(($completedBuilds / $totalBuilds) * 100)
            Write-Progress -Activity "Building $pkg for $($plat.name)" -PercentComplete $progress

            Write-Host "  Building $pkg for $($plat.name) ($($plat.target))..." -NoNewline

            try {
                $result = cargo build --release --target $plat.target -p $pkg 2>&1
                $success = $LASTEXITCODE -eq 0

                if ($Verbose) {
                    Write-Host ""
                    Write-Host $result
                }

                if ($success) {
                    Write-Host " OK" -ForegroundColor $global:ScriptColors.Ok
                } else {
                    Write-Host " FAILED" -ForegroundColor $global:ScriptColors.Err
                    if (-not $Verbose) {
                        Write-Host $result
                    }
                    $failedBuilds += "$pkg@$($plat.name)"
                }
            } catch {
                Write-Host " ERROR" -ForegroundColor $global:ScriptColors.Err
                Write-Error "Build failed for $pkg on $($plat.name): $($_.Exception.Message)"
                $failedBuilds += "$pkg@$($plat.name)"
            }
        }
    }

    Clear-Progress
}

$buildDuration = Stop-Timer -Timer $buildTimer
$durationStr = Format-Duration -Duration $buildDuration

Write-Host ""
Write-Step "Build summary"
Write-Host ""

if ($failedBuilds.Count -eq 0) {
    Write-Ok "All builds successful"
    Write-Muted "Duration: $durationStr"
    Write-Host ""
    Write-Host "  Binaries in target/{target}/release/" -ForegroundColor $global:ScriptColors.Info
    exit 0
} else {
    Write-Error "Failed builds:"
    foreach ($build in $failedBuilds) {
        Write-Muted "  - $build"
    }
    Write-Muted "Duration: $durationStr"
    exit 1
}
