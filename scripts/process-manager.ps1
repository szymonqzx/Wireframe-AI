#!/usr/bin/env pwsh
# Wireframe-AI Process Manager
# Handles module rebuilds and restarts for selfdev mode

param(
    [Parameter(Mandatory=$false)]
    [string]$Module,

    [Parameter(Mandatory=$false)]
    [string]$OldModule,

    [Parameter(Mandatory=$false)]
    [string]$NewModule,

    [Parameter(Mandatory=$false)]
    [string]$NatsUrl = "nats://localhost:4222",

    [Parameter(Mandatory=$false)]
    [switch]$AutoRestart,

    [Parameter(Mandatory=$false)]
    [switch]$Switch,

    [Parameter(Mandatory=$false)]
    [switch]$Force
)

$ErrorActionPreference = "Stop"

function Log-Message {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
}

function Send-NatsMessage {
    param(
        [string]$Topic,
        [string]$Payload
    )

    try {
        # Use nats CLI if available, otherwise skip
        if (Get-Command nats -ErrorAction SilentlyContinue) {
            nats pub $Topic $Payload -s $NatsUrl | Out-Null
            Log-Message "Published to $Topic"
        } else {
            Log-Message "NATS CLI not found, skipping publish to $Topic" "WARN"
        }
    } catch {
        Log-Message "Failed to publish to $Topic: $_" "ERROR"
    }
}

function Get-ModuleBinaryPath {
    param([string]$Module)

    $moduleMap = @{
        "wireframe-adapter-rust" = "target/release/wireframe-adapter.exe"
        "wireframe-ai-context-core" = "target/release/wireframe-ai-context-core.exe"
        "wireframe-ai-orchestrator-core" = "target/release/wireframe-ai-orchestrator-core.exe"
        "wireframe-ai-sandbox-core" = "target/release/wireframe-ai-sandbox-core.exe"
        "wireframe-ai-interface" = "target/release/wireframe-interface.exe"
    }

    if ($moduleMap.ContainsKey($Module)) {
        return $moduleMap[$Module]
    }

    throw "Unknown module: $Module"
}

function Get-ProcessId {
    param([string]$Module)

    $processName = $Module -replace "^wireframe-|^wireframe-ai-", ""
    $process = Get-Process -Name $processName -ErrorAction SilentlyContinue

    if ($process) {
        return $process.Id
    }

    return $null
}

function Stop-Module {
    param(
        [string]$Module,
        [int]$ProcessId
    )

    Log-Message "Stopping module $Module (PID: $ProcessId)"

    try {
        Stop-Process -Id $ProcessId -Force
        Log-Message "Module $Module stopped"
        return $true
    } catch {
        Log-Message "Failed to stop module $Module: $_" "ERROR"
        return $false
    }
}

function Start-Module {
    param(
        [string]$Module,
        [string]$BinaryPath
    )

    Log-Message "Starting module $Module from $BinaryPath"

    try {
        $process = Start-Process -FilePath $BinaryPath -PassThru
        Log-Message "Module $Module started (PID: $($process.Id))"
        return $process.Id
    } catch {
        Log-Message "Failed to start module $Module: $_" "ERROR"
        return $null
    }
}

function Build-Module {
    param([string]$Module)

    Log-Message "Building module $Module"

    try {
        $result = cargo build --release -p $Module
        if ($LASTEXITCODE -eq 0) {
            Log-Message "Build successful for $Module"
            return $true
        } else {
            Log-Message "Build failed for $Module" "ERROR"
            return $false
        }
    } catch {
        Log-Message "Build error for $Module: $_" "ERROR"
        return $false
    }
}

function Switch-Module {
    param(
        [string]$OldModule,
        [string]$NewModule,
        [bool]$Force
    )

    Log-Message "Switching from $OldModule to $NewModule (force: $Force)"

    # Get binary paths
    $oldBinaryPath = Get-ModuleBinaryPath -Module $OldModule
    $newBinaryPath = Get-ModuleBinaryPath -Module $NewModule

    Log-Message "Old binary: $oldBinaryPath"
    Log-Message "New binary: $newBinaryPath"

    # Get current process ID
    $oldPid = Get-ProcessId -Module $OldModule
    if ($oldPid) {
        Log-Message "Current process ID: $oldPid"
    } else {
        Log-Message "No running process found for $OldModule" "WARN"
    }

    # Stop old module
    if ($oldPid) {
        $stopSuccess = Stop-Module -Module $OldModule -ProcessId $oldPid
        if (-not $stopSuccess) {
            Log-Message "Failed to stop old module" "ERROR"
            return $false
        }

        # Wait for process to stop
        Start-Sleep -Seconds 2
    }

    # Start new module
    $newPid = Start-Module -Module $NewModule -BinaryPath $newBinaryPath
    if ($newPid) {
        Log-Message "Switch successful. New PID: $newPid"

        # Send acknowledgment
        $ackPayload = @{
            request_id = "switch-$((Get-Date -Format yyyyMMddHHmmss))"
            status = "Completed"
            message = "Switch completed successfully"
            old_pid = $oldPid
            new_pid = $newPid
            timestamp = [int][double]::Parse((Get-Date -UFormat %s))
        } | ConvertTo-Json

        Send-NatsMessage -Topic "module.switch.ack" -Payload $ackPayload
        return $true
    } else {
        Log-Message "Failed to start new module" "ERROR"

        # Rollback: try to restart old module
        if ($oldPid) {
            Log-Message "Attempting rollback to $OldModule"
            $rollbackPid = Start-Module -Module $OldModule -BinaryPath $oldBinaryPath
            if ($rollbackPid) {
                Log-Message "Rollback successful. PID: $rollbackPid"
            }
        }

        return $false
    }
}

# Main execution
if ($Switch) {
    if (-not $OldModule -or -not $NewModule) {
        Log-Message "Switch mode requires -OldModule and -NewModule parameters" "ERROR"
        exit 1
    }

    Log-Message "Process Manager starting in switch mode"
    Log-Message "Switching from $OldModule to $NewModule"

    try {
        $switchSuccess = Switch-Module -OldModule $OldModule -NewModule $NewModule -Force $Force
        if ($switchSuccess) {
            Log-Message "Module switch completed successfully"
            exit 0
        } else {
            Log-Message "Module switch failed" "ERROR"
            exit 1
        }
    } catch {
        Log-Message "Switch error: $_" "ERROR"
        exit 1
    }
}

Log-Message "Process Manager starting for module: $Module"

try {
    # Get binary path
    $binaryPath = Get-ModuleBinaryPath -Module $Module
    Log-Message "Binary path: $binaryPath"

    # Subscribe to module.restart.request (simplified - in production use NATS client)
    Log-Message "Listening for restart requests on module.restart.$Module"

    if ($AutoRestart) {
        # Auto-restart mode: build and restart immediately
        Log-Message "Auto-restart mode enabled"

        # Get current process ID
        $currentPid = Get-ProcessId -Module $Module
        if ($currentPid) {
            Log-Message "Current process ID: $currentPid"
        } else {
            Log-Message "No running process found for $Module" "WARN"
        }

        # Build module
        $buildSuccess = Build-Module -Module $Module
        if (-not $buildSuccess) {
            Log-Message "Build failed, aborting restart" "ERROR"
            exit 1
        }

        # Stop current process if running
        if ($currentPid) {
            $stopSuccess = Stop-Module -Module $Module -ProcessId $currentPid
            if (-not $stopSuccess) {
                Log-Message "Failed to stop current process" "ERROR"
                exit 1
            }

            # Wait for process to stop
            Start-Sleep -Seconds 2
        }

        # Start new process
        $newPid = Start-Module -Module $Module -BinaryPath $binaryPath
        if ($newPid) {
            Log-Message "Restart successful. New PID: $newPid"

            # Send acknowledgment
            $ackPayload = @{
                module_name = $Module
                request_id = "auto-$((Get-Date -Format yyyyMMddHHmmss))"
                status = "completed"
                message = "Auto-restart completed"
                new_pid = $newPid
                old_pid = $currentPid
                timestamp = [int][double]::Parse((Get-Date -UFormat %s))
            } | ConvertTo-Json

            Send-NatsMessage -Topic "module.restart.ack" -Payload $ackPayload
        } else {
            Log-Message "Failed to start new process" "ERROR"
            exit 1
        }
    } else {
        # Interactive mode: wait for manual restart requests
        Log-Message "Interactive mode. Press Ctrl+C to exit."
        Log-Message "Use: .\scripts\process-manager.ps1 -Module $Module -AutoRestart to auto-restart"
    }

} catch {
    Log-Message "Process manager error: $_" "ERROR"
    exit 1
}

Log-Message "Process manager exiting"
