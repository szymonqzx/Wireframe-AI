#!/usr/bin/env pwsh
# TEAM_006: Start a local demo: NATS + context + orchestrator with enhanced error handling
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

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - Demo Launcher               |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

$natsBin = Join-Path $PSScriptRoot ".." "kernel" "nats" "nats-server.exe"

# Check if NATS is running
$natsRunning = Test-ProcessRunning -ProcessName "nats-server"

Write-Step "Starting demo environment..."

if (-not $natsRunning) {
    if (Test-Path $natsBin) {
        Write-Host "     ℹ Starting NATS server..." -ForegroundColor $global:ScriptColors.Info
        try {
            $natsProc = Start-TrackedProcess -Name "nats-server" -Path $natsBin -Hidden
            Start-Sleep -Seconds 2

            if ($natsProc -and -not $natsProc.HasExited) {
                Write-Ok "NATS server started (PID: $($natsProc.Id))"
                Register-Cleanup -Handler {
                    Stop-TrackedProcess -Name "nats-server"
                    Write-InfoMsg "Stopped NATS server"
                }
            } else {
                Write-Error "NATS server failed to start"
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
}

Write-Step "Starting modules..."

# Start context module
Write-InfoMsg "Starting context module..."
$contextJob = Start-Job -ScriptBlock {
    param($dir)
    cd $dir
    cargo run --release -p wireframe-ai-context-core 2>&1
} -ArgumentList (Get-Location).Path

Start-Sleep -Seconds 2

# Check context started
$contextRunning = $null -ne (Get-Job -Id $contextJob.Id | Where-Object State -eq "Running")
if (-not $contextRunning) {
    $output = Receive-Job -Id $contextJob.Id
    Write-Error "Context module failed to start"
    if ($Verbose) {
        Write-Host $output
    }
    exit 1
}
Write-Ok "Context module started (job: $($contextJob.Id))"

Register-Cleanup -Handler {
    Stop-Job -Id $contextJob.Id -ErrorAction SilentlyContinue
    Remove-Job -Id $contextJob.Id -Force -ErrorAction SilentlyContinue
    Write-InfoMsg "Stopped context module"
}

# Start orchestrator
Write-InfoMsg "Starting orchestrator..."
$orchJob = Start-Job -ScriptBlock {
    param($dir)
    cd $dir
    cargo run --release -p wireframe-ai-orchestrator-core 2>&1
} -ArgumentList (Get-Location).Path

Start-Sleep -Seconds 2

# Check orchestrator started
$orchRunning = $null -ne (Get-Job -Id $orchJob.Id | Where-Object State -eq "Running")
if (-not $orchRunning) {
    $output = Receive-Job -Id $orchJob.Id
    Write-Warn "Orchestrator failed to start (continuing without it)"
    if ($Verbose) {
        Write-Host $output
    }
} else {
    Write-Ok "Orchestrator started (job: $($orchJob.Id))"

    Register-Cleanup -Handler {
        Stop-Job -Id $orchJob.Id -ErrorAction SilentlyContinue
        Remove-Job -Id $orchJob.Id -Force -ErrorAction SilentlyContinue
        Write-InfoMsg "Stopped orchestrator"
    }
}

Write-Host ""
Write-Step "Demo running!"
Write-Host ""
Write-Muted "Use Ctrl+C to stop all modules"
Write-Muted "In another terminal, run: cargo run --release -p wireframe-ai-interface 'your task here'"
Write-Host ""

try {
    while ($true) {
        Start-Sleep -Seconds 1
    }
} finally {
    Write-Host ""
    Write-Step "Shutting down..."
    Invoke-Cleanup
    Write-Ok "All modules stopped"
    Write-Host ""
}
