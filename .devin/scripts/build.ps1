#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Build automation script for Wireframe-AI project.

.DESCRIPTION
    This script provides automated build options including release builds,
    debug builds, and build verification for all Wireframe-AI modules.

.PARAMETER Release
    Build in release mode. Default: false

.PARAMETER Clean
    Clean build artifacts before building. Default: false

.PARAMETER Verify
    Run build verification tests after build. Default: false

.PARAMETER Target
    Target triple for cross-compilation (e.g., x86_64-pc-windows-msvc)

.PARAMETER Package
    Build specific package only (e.g., wireframe-ai-context)
#>

param(
    [Parameter()]
    [switch]$Release,

    [Parameter()]
    [switch]$Clean,

    [Parameter()]
    [switch]$Verify,

    [Parameter()]
    [string]$Target,

    [Parameter()]
    [string]$Package
)

$ErrorActionPreference = "Stop"

$packages = @(
    "wireframe-ai-interface",
    "wireframe-ai-context",
    "wireframe-ai-orchestrator",
    "wireframe-ai-sandbox",
    "wireframe-tui-runner"
)

function Invoke-CleanBuild {
    Write-Host "Cleaning build artifacts..."
    cargo clean
}

function Invoke-Build {
    param(
        [switch]$IsRelease,
        [string]$BuildTarget,
        [string]$BuildPackage
    )

    $buildArgs = @("build")

    if ($IsRelease) {
        $buildArgs += "--release"
    }

    if ($BuildTarget) {
        $buildArgs += "--target", $BuildTarget
    }

    if ($BuildPackage) {
        $buildArgs += "-p", $BuildPackage
    }

    Write-Host "Building with args: $($buildArgs -join ' ')"
    & cargo @buildArgs

    if ($LASTEXITCODE -ne 0) {
        throw "Build failed with exit code $LASTEXITCODE"
    }
}

function Invoke-BuildVerification {
    param([switch]$IsRelease)

    Write-Host "Running build verification..."

    $targetDir = if ($IsRelease) { "release" } else { "debug" }
    $binaryDir = "target/$targetDir"

    if (-not (Test-Path $binaryDir)) {
        throw "Build verification failed: Binary directory not found at $binaryDir"
    }

    $expectedBinaries = @(
        "wireframe-ai-interface.exe",
        "wireframe-ai-context.exe",
        "wireframe-ai-orchestrator.exe",
        "wireframe-ai-sandbox.exe"
    )

    $missingBinaries = @()
    foreach ($binary in $expectedBinaries) {
        $binaryPath = Join-Path $binaryDir $binary
        if (-not (Test-Path $binaryPath)) {
            $missingBinaries += $binary
        }
    }

    if ($missingBinaries.Count -gt 0) {
        throw "Build verification failed: Missing binaries: $($missingBinaries -join ', ')"
    }

    Write-Host "Build verification passed - all binaries found"
}

function Get-BuildInfo {
    Write-Host "=== Build Information ==="

    $rustcVersion = rustc --version 2>$null
    $cargoVersion = cargo --version 2>$null

    Write-Host "Rustc: $rustcVersion"
    Write-Host "Cargo: $cargoVersion"

    if ($Target) {
        Write-Host "Target: $Target"
    }

    if ($Package) {
        Write-Host "Package: $Package"
    } else {
        Write-Host "Packages: $($packages -join ', ')"
    }

    Write-Host ""
}

Write-Host "=== Wireframe-AI Build Automation ==="
Write-Host ""

Get-BuildInfo

if ($Clean) {
    Invoke-CleanBuild
}

if (-not $Release) {
    Write-Host "Building debug..."
    Invoke-Build -IsRelease:$false -BuildTarget $Target -BuildPackage $Package
}

if ($Release) {
    Write-Host "Building release..."
    Invoke-Build -IsRelease -BuildTarget $Target -BuildPackage $Package
}

if ($Verify) {
    Invoke-BuildVerification -IsRelease:$Release
}

Write-Host ""
Write-Host "Build complete!"
