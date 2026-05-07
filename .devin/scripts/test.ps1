#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Test automation script for Wireframe-AI project.

.DESCRIPTION
    This script provides automated test execution including unit tests,
    integration tests, with coverage, and test result reporting.

.PARAMETER Unit
    Run unit tests. Default: true

.PARAMETER Integration
    Run integration tests. Default: true

.PARAMETER Coverage
    Run tests with coverage. Default: false

.PARAMETER Verbose
    Enable verbose test output. Default: false

.PARAMETER NoCapture
    Disable output capture (show stdout/stderr). Default: false

.PARAMETER TestFilter
    Filter tests by name pattern

.PARAMETER NoFailFast
    Continue running tests after first failure. Default: false
#>

param(
    [Parameter()]
    [switch]$Unit,

    [Parameter()]
    [switch]$Integration,

    [Parameter()]
    [switch]$Coverage,

    [Parameter()]
    [switch]$Verbose,

    [Parameter()]
    [switch]$NoCapture,

    [Parameter()]
    [string]$TestFilter,

    [Parameter()]
    [switch]$NoFailFast
)

$ErrorActionPreference = "Stop"

function Invoke-UnitTests {
    param(
        [switch]$Coverage,
        [switch]$Verbose,
        [switch]$NoCapture,
        [string]$TestFilter,
        [switch]$NoFailFast
    )

    Write-Host "Running unit tests..."

    $testArgs = @("test", "--lib")

    if ($Verbose) {
        $testArgs += "--", "--nocapture"
    }

    if ($NoCapture) {
        $testArgs += "--", "--nocapture"
    }

    if ($TestFilter) {
        $testArgs += "--", $TestFilter
    }

    if ($NoFailFast) {
        $testArgs += "--no-fail-fast"
    }

    & cargo @testArgs

    if ($LASTEXITCODE -ne 0) {
        throw "Unit tests failed with exit code $LASTEXITCODE"
    }

    Write-Host "✓ Unit tests passed"
}

function Invoke-IntegrationTests {
    param(
        [switch]$Coverage,
        [switch]$Verbose,
        [switch]$NoCapture,
        [string]$TestFilter,
        [switch]$NoFailFast
    )

    Write-Host "Running integration tests..."

    $testArgs = @("test", "--test", "integration_test")

    if ($Verbose) {
        $testArgs += "--", "--nocapture"
    }

    if ($NoCapture) {
        $testArgs += "--", "--nocapture"
    }

    if ($TestFilter) {
        $testArgs += "--", $TestFilter
    }

    if ($NoFailFast) {
        $testArgs += "--no-fail-fast"
    }

    & cargo @testArgs

    if ($LASTEXITCODE -ne 0) {
        throw "Integration tests failed with exit code $LASTEXITCODE"
    }

    Write-Host "✓ Integration tests passed"
}

function Get-TestSummary {
    Write-Host ""
    Write-Host "=== Test Summary ==="
    Write-Host "All tests passed successfully!"
}

# Main execution
Write-Host "=== Wireframe-AI Test Automation ==="
Write-Host ""

if ($Unit -or (-not $Integration)) {
    Invoke-UnitTests -Coverage:$Coverage -Verbose:$Verbose -NoCapture:$NoCapture -TestFilter $TestFilter -NoFailFast:$NoFailFast
}

if ($Integration) {
    Invoke-IntegrationTests -Coverage:$Coverage -Verbose:$Verbose -NoCapture:$NoCapture -TestFilter $TestFilter -NoFailFast:$NoFailFast
}

Get-TestSummary
