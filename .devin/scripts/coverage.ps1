#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Run code coverage analysis for Wireframe-AI project.

.DESCRIPTION
    This script runs code coverage using tarpaulin or grcov and generates reports.
    Supports multiple output formats (lcov, html, cobertura, json).

.PARAMETER Tool
    Coverage tool to use: tarpaulin or grcov. Default: tarpaulin

.PARAMETER OutputFormat
    Output format: lcov, html, cobertura, json, or all. Default: html

.PARAMETER Threshold
    Minimum coverage percentage threshold (0-100). Default: 80

.PARAMETER OpenReport
    Open the coverage report in browser after generation. Default: true

.PARAMETER Clean
    Clean previous coverage data before running. Default: true
#>

param(
    [Parameter()]
    [ValidateSet("tarpaulin", "grcov")]
    [string]$Tool = "tarpaulin",

    [Parameter()]
    [ValidateSet("lcov", "html", "cobertura", "json", "all")]
    [string]$OutputFormat = "html",

    [Parameter()]
    [ValidateRange(0, 100)]
    [int]$Threshold = 80,

    [Parameter()]
    [switch]$OpenReport,

    [Parameter()]
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

$CoverageDir = "coverage"
$ReportDir = "$CoverageDir/reports"

function Test-CoverageTool {
    param([string]$ToolName)

    $null = Get-Command $ToolName -ErrorAction SilentlyContinue
    return $?
}

function Initialize-CoverageEnvironment {
    Write-Host "Initializing coverage environment..."

    if ($Clean -and (Test-Path $CoverageDir)) {
        Write-Host "Cleaning previous coverage data..."
        Remove-Item -Recurse -Force $CoverageDir
    }

    New-Item -ItemType Directory -Force -Path $ReportDir | Out-Null
}

function Invoke-TarpaulinCoverage {
    Write-Host "Running tarpaulin coverage..."

    $tarpaulinArgs = @(
        "--out", $OutputFormat,
        "--output-dir", $ReportDir,
        "--verbose"
    )

    if ($OutputFormat -eq "all") {
        $tarpaulinArgs = @(
            "--out", "Html", "--out", "Lcov",
            "--out", "Xml",
            "--output-dir", $ReportDir,
            "--verbose"
        )
    }

    & cargo tarpaulin $tarpaulinArgs

    if ($LASTEXITCODE -ne 0) {
        throw "Tarpaulin coverage failed with exit code $LASTEXITCODE"
    }
}

function Invoke-GrcovCoverage {
    Write-Host "Running grcov coverage..."

    # First, run tests with coverage instrumentation
    $env:CARGO_INCREMENTAL = "0"
    $env:RUSTFLAGS = "-Cinstrument-coverage"
    $env:LLVM_PROFILE_FILE = "coverage-%p-%m.profraw"

    Write-Host "Building with coverage instrumentation..."
    cargo build

    Write-Host "Running tests..."
    cargo test

    # Generate coverage report with grcov
    $grcovArgs = @(
        ".",
        "--binary-path", "./target/debug/deps",
        "-s", ".",
        "-t", $OutputFormat,
        "--branch",
        "--ignore-not-existing",
        "--ignore", "/*",
        "-o", "$ReportDir/coverage.$OutputFormat"
    )

    & grcov $grcovArgs

    if ($LASTEXITCODE -ne 0) {
        throw "Grcov coverage failed with exit code $LASTEXITCODE"
    }
}

function Get-CoveragePercentage {
    param([string]$ReportFile)

    if (-not (Test-Path $ReportFile)) {
        Write-Warning "Coverage report not found: $ReportFile"
        return $null
    }

    # Parse lcov report for coverage percentage
    if ($ReportFile -match "\.lcov$") {
        $content = Get-Content $ReportFile -Raw
        if ($content -match "LF:(\d+).*?LH:(\d+)") {
            $linesFound = [int]$matches[1]
            $linesHit = [int]$matches[2]
            if ($linesFound -gt 0) {
                return [math]::Round(($linesHit / $linesFound) * 100, 2)
            }
        }
    }

    return $null
}

function Test-CoverageThreshold {
    param([int]$Coverage, [int]$Threshold)

    if ($Coverage -lt $Threshold) {
        Write-Error "Coverage ($Coverage%) is below threshold ($Threshold%)"
        return $false
    }

    Write-Host "✓ Coverage ($Coverage%) meets threshold ($Threshold%)"
    return $true
}

function Open-CoverageReport {
    param([string]$Format)

    $reportFile = switch ($Format) {
        "html" { "$ReportDir/tarpaulin-report.html" }
        "lcov" { "$ReportDir/lcov.info" }
        "cobertura" { "$ReportDir/cobertura.xml" }
        "json" { "$ReportDir/coverage.json" }
        "all" { "$ReportDir/tarpaulin-report.html" }
        default { "$ReportDir/coverage.$Format" }
    }

    if (Test-Path $reportFile) {
        Write-Host "Opening coverage report: $reportFile"
        Start-Process $reportFile
    } else {
        Write-Warning "Report file not found: $reportFile"
    }
}

# Main execution
Write-Host "=== Code Coverage Analysis ==="
Write-Host "Tool: $Tool"
Write-Host "Output: $OutputFormat"
Write-Host "Threshold: $Threshold%"
Write-Host ""

# Check if coverage tool is installed
if (-not (Test-CoverageTool $Tool)) {
    Write-Error "Coverage tool '$Tool' is not installed. Please install it first."
    Write-Host "Install tarpaulin: cargo install cargo-tarpaulin"
    Write-Host "Install grcov: cargo install grcov"
    exit 1
}

Initialize-CoverageEnvironment

# Run coverage based on selected tool
switch ($Tool) {
    "tarpaulin" { Invoke-TarpaulinCoverage }
    "grcov" { Invoke-GrcovCoverage }
}

# Calculate coverage percentage
$coverage = Get-CoveragePercentage -ReportFile "$ReportDir/lcov.info"

if ($coverage) {
    Write-Host ""
    Write-Host "Coverage: $coverage%"

    # Test against threshold
    $meetsThreshold = Test-CoverageThreshold -Coverage $coverage -Threshold $Threshold
    if (-not $meetsThreshold) {
        exit 1
    }
}

# Open report if requested
if ($OpenReport) {
    Open-CoverageReport -Format $OutputFormat
}

Write-Host ""
Write-Host "Coverage analysis complete!"
