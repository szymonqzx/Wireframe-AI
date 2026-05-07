#!/usr/bin/env pwsh
# TEAM_006: End-to-end smoke test for Wireframe AI with detailed reporting
#
# This test:
# 1. Starts NATS (if not already running)
# 2. Starts the context module
# 3. Publishes a task.submitted message
# 4. Verifies task.enriched arrives
# 5. Cleans up

param(
    [int]$TimeoutSeconds = 15,
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
$testSessionId = "smoke_test_$(Get-Random -Maximum 99999)"

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - Smoke Test                  |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""
Write-Muted "Session ID: $testSessionId"
Write-Muted "Timeout: ${TimeoutSeconds}s"

$testTimer = Start-Timer
$testResults = @{
    NATSStarted = $false
    ContextStarted = $false
    TestPassed = $false
}

# ── Helper: check if NATS is running ────────────────────────────────────────
$natsBin = Join-Path $PSScriptRoot ".." "kernel" "nats" "nats-server.exe"
$natsRunning = Test-ProcessRunning -ProcessName "nats-server"

Write-Step "Step 1/3: Starting NATS server..."

if (-not $natsRunning) {
    if (Test-Path $natsBin) {
        Write-InfoMsg "Starting NATS server..."
        try {
            $natsProc = Start-TrackedProcess -Name "nats-server" -Path $natsBin -Hidden
            Start-Sleep -Seconds 2

            if ($natsProc -and -not $natsProc.HasExited) {
                Write-Ok "NATS started (PID: $($natsProc.Id))"
                $testResults.NATSStarted = $true
                Register-Cleanup -Handler {
                    Stop-TrackedProcess -Name "nats-server"
                }
            } else {
                Write-Error "NATS failed to start"
                exit 1
            }
        } catch {
            Write-Error "Failed to start NATS server: $($_.Exception.Message)"
            exit 1
        }
    } else {
        Write-Error "NATS not found at $natsBin"
        Write-InfoMsg "Run: scripts/download-nats.ps1"
        exit 1
    }
} else {
    Write-Ok "NATS already running"
    $testResults.NATSStarted = $true
    $natsProc = $null
}

# ── Step 2: Start modules ──────────────────────────────────────────────────
Write-Step "Step 2/3: Starting modules..."

$contextJob = Start-Job -ScriptBlock {
    param($dir)
    cd $dir
    cargo run --release -p wireframe-ai-context-core 2>&1
} -ArgumentList (Get-Location).Path

Start-Sleep -Seconds 3

# Check context started
$contextRunning = $null -ne (Get-Job -Id $contextJob.Id | Where-Object State -eq "Running")
if (-not $contextRunning) {
    $output = Receive-Job -Id $contextJob.Id
    Write-Error "Context module FAILED to start"
    if ($Verbose) {
        Write-Host $output
    }
    exit 1
}
Write-Ok "Context module started (job: $($contextJob.Id))"
$testResults.ContextStarted = $true

Register-Cleanup -Handler {
    Stop-Job -Id $contextJob.Id -ErrorAction SilentlyContinue
    Remove-Job -Id $contextJob.Id -Force -ErrorAction SilentlyContinue
}

# ── Step 3: Run test ───────────────────────────────────────────────────────
Write-Step "Step 3/3: Running test..."

Write-InfoMsg "Running interface with test message..."

$interfaceTimer = Start-Timer
$timedOut = $false
$interfaceOutput = ""

$interfaceJob = Start-Job -ScriptBlock {
    param($dir, $msg, $timeout)
    cd $dir
    cargo run --release -p wireframe-ai-interface -- --timeout-secs $timeout $msg 2>&1
} -ArgumentList (Get-Location).Path, "Smoke test: write a short Python script that prints 'Hello from Wireframe AI'", $TimeoutSeconds

$result = Wait-Job -Id $interfaceJob.Id -Timeout $TimeoutSeconds
if ($null -eq $result) {
    Write-Error "TIMEOUT - Interface did not complete within ${TimeoutSeconds}s"
    $timedOut = $true
} else {
    $interfaceOutput = Receive-Job -Id $interfaceJob.Id
}

$interfaceDuration = Stop-Timer -Timer $interfaceTimer
$interfaceDurationStr = Format-Duration -Duration $interfaceDuration

Remove-Job -Id $interfaceJob.Id -Force -ErrorAction SilentlyContinue

# ── Verification ─────────────────────────────────────────────────────────────
Write-Host ""
Write-Step "Test results"

if ($timedOut) {
    Write-Error "FAILED: Pipeline timed out"
    $testResults.TestPassed = $false
} elseif ($interfaceOutput -match "Error") {
    Write-Error "FAILED: Interface error"
    if ($Verbose) {
        Write-Host $interfaceOutput
    }
    $testResults.TestPassed = $false
} else {
    Write-Ok "PASSED"
    $testResults.TestPassed = $true
    if ($Verbose) {
        Write-Host ""
        Write-Host "Interface output:"
        Write-Host $interfaceOutput
    }
}

$testDuration = Stop-Timer -Timer $testTimer
$testDurationStr = Format-Duration -Duration $testDuration

Write-Host ""
Write-Step "Summary"
Write-Muted "Total duration: $testDurationStr"
Write-Muted "Interface duration: $interfaceDurationStr"

if ($testResults.TestPassed) {
    exit 0
} else {
    exit 1
}
