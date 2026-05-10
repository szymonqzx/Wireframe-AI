#!/usr/bin/env pwsh
# TEAM_006: Start all Wireframe AI modules in separate cmd windows with enhanced error handling
# Opens the interface (the "UI") at the end for user input.
#
# Usage:
#   .\scripts\start-all.ps1                    # default mode (release)
#   .\scripts\start-all.ps1 -BuildMode debug   # debug mode
#   .\scripts\start-all.ps1 -SkipOrchestrator  # without the orchestrator
#   .\scripts\start-all.ps1 -SkipAdapter       # without the Python adapter
#
# Each module runs in its own cmd window, title-bar labeled so you can
# quickly spot which is which. Ctrl+C in the interface window exits cleanly.

param(
    [ValidateSet("release", "debug")]
    [string]$BuildMode = "debug",

    [switch]$SkipOrchestrator,
    [switch]$SkipAdapter,
    [switch]$SkipBuild,
    [switch]$psmux,
    [switch]$Help,
    [switch]$Verbose
)

if ($Help) {
    Get-Help $PSCommandPath -Detailed
    exit 0
}

# Load common utilities
$commonPath = Join-Path $PSScriptRoot "common.ps1"
if (Test-Path $commonPath) {
    . $commonPath
} else {
    Write-Error "common.ps1 not found at $commonPath"
    exit 1
}

$ErrorActionPreference = "Stop"

# ── Paths ────────────────────────────────────────────────────────────────────
$RootDir = Resolve-Path "$PSScriptRoot/.."
$NatsBin = Join-Path (Join-Path (Join-Path $RootDir "kernel") "nats") "nats-server.exe"
$TuiPath = Join-Path $RootDir "tools\tui-minimal"
$InterfacePath = Join-Path $RootDir "kernel\interface"

$BuildFlag = if ($BuildMode -eq "release") { "--release" } else { "" }
$BuildLabel = if ($BuildMode -eq "release") { "release" } else { "debug" }

# Track what we started so we can clean up
# Note: We use the common.ps1 tracked process functions instead

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |        Wireframe AI - Launch All Modules         |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

# ── Preflight checks ─────────────────────────────────────────────────────────
Write-Step "Checking prerequisites..."

# Check NATS — download if missing
$natsExisted = Test-Path $NatsBin
if (-not $natsExisted) {
    Write-Warn "NATS binary not found at kernel/nats/nats-server.exe"
    Write-InfoMsg "Downloading NATS server..."
    try {
        Invoke-WithRetry -MaxRetries 3 -OperationName "NATS download" -ScriptBlock {
            & "$PSScriptRoot/download-nats.ps1" 2>&1 | Out-Null
        }
        $natsExisted = Test-Path $NatsBin
    } catch {
        Write-Warn "Failed to download NATS automatically: $($_.Exception.Message)"
    }
}
if ($natsExisted) {
    Write-Ok "NATS binary found at kernel/nats/nats-server.exe"
} else {
    Write-Warn "Failed to download NATS automatically."
    Write-Warn "Start NATS manually (docker run -p 4222:4222 nats:latest) and re-run."
    Write-Warn "Continuing anyway - modules will fail to connect."
}

# Check Rust
if (-not (Test-Command "cargo")) {
    Write-Error "Rust toolchain (cargo) not found. Install from https://rustup.rs"
    exit 1
}
Write-Ok "Rust toolchain found"

# Check Python + install packages (only if adapter is enabled)
if (-not $SkipAdapter) {
    $python = Get-Command "python" -ErrorAction SilentlyContinue
    if (-not $python) {
        Write-Warn "Python not found - adapter will be skipped"
        $SkipAdapter = $true
    } else {
        Write-Ok "Python found"

        # Install SDK + adapter packages (idempotent — pip skips if already installed)
        $sdkDir = Join-Path (Join-Path $RootDir "sdk") "agentic-sdk-py"
        $adapterDir = Join-Path (Join-Path $RootDir "adapter") "python"

        Push-Location $RootDir
        try {
            Write-InfoMsg "Installing agentic-sdk-py..."
            cmd /c "pip install -e `"$sdkDir`" --quiet >nul 2>&1"
            if ($LASTEXITCODE -eq 0) {
                Write-Ok "agentic-sdk-py ready"
            } else {
                Write-Warn "Failed to install agentic-sdk-py"
            }

            Write-InfoMsg "Installing wireframe-ai-adapter..."
            cmd /c "pip install -e `"$adapterDir`" --quiet >nul 2>&1"
            if ($LASTEXITCODE -eq 0) {
                Write-Ok "wireframe-ai-adapter ready"
            } else {
                Write-Warn "Failed to install wireframe-ai-adapter"
            }
        } finally {
            Pop-Location
        }
    }
}

# Check for psmux (PowerShell multiplexer)
$psmuxAvailable = $false
$psmuxCmdName = $null
try {
    # Check for pmux first (preferred CLI interface)
    $pmuxCmd = Get-Command "pmux" -ErrorAction SilentlyContinue
    if ($pmuxCmd) {
        $psmuxAvailable = $true
        $psmuxCmdName = "pmux"
        Write-Ok "pmux found - modules will open in panes"
    } else {
        # Fall back to psmux
        $psmuxCmd = Get-Command "psmux" -ErrorAction SilentlyContinue
        if ($psmuxCmd) {
            $psmuxAvailable = $true
            $psmuxCmdName = "psmux"
            Write-Ok "psmux found - modules will open in panes"
        } else {
            if ($psmux) {
                Write-Step "Installing psmux via cargo..."
                try {
                    cargo install psmux
                    if ($LASTEXITCODE -eq 0) {
                        # Check which command got installed
                        $pmuxCmd = Get-Command "pmux" -ErrorAction SilentlyContinue
                        if ($pmuxCmd) {
                            $psmuxCmdName = "pmux"
                        } else {
                            $psmuxCmdName = "psmux"
                        }
                        $psmuxAvailable = $true
                        Write-Ok "psmux installed successfully - modules will open in panes"
                    } else {
                        throw "cargo install failed with exit code $LASTEXITCODE"
                    }
                } catch {
                    Write-Warn "Failed to install psmux: $($_.Exception.Message)"
                    Write-Warn "Modules will open in separate windows"
                }
            } else {
                Write-Muted "psmux not found - modules will open in separate windows"
                Write-Muted "Use -psmux to install psmux automatically"
            }
        }
    }
} catch {
    Write-Muted "psmux not available - modules will open in separate windows"
}

# ── API Key Configuration ───────────────────────────────────────────────────────
if (-not $SkipAdapter) {
    Write-Step "API Key Configuration"
    Write-Host ""

    $keysConfigured = $false

    # Check if keys are already set
    if ($env:OPENAI_API_KEY -or $env:ANTHROPIC_API_KEY -or $env:DEEPSEEK_API_KEY -or $env:OPENCODE_GO_API_KEY) {
        Write-Ok "API keys already configured in environment"
        if ($env:OPENAI_API_KEY) { Write-Muted "  OPENAI_API_KEY: ***" }
        if ($env:ANTHROPIC_API_KEY) { Write-Muted "  ANTHROPIC_API_KEY: ***" }
        if ($env:DEEPSEEK_API_KEY) { Write-Muted "  DEEPSEEK_API_KEY: ***" }
        if ($env:OPENCODE_GO_API_KEY) { Write-Muted "  OPENCODE_GO_API_KEY: ***" }
        $keysConfigured = $true
    }

    if (-not $keysConfigured) {
        Write-Host "  Configure API keys for LLM providers (press Enter to skip):" -ForegroundColor $global:ScriptColors.Info
        Write-Host ""

        $openaiKey = Read-Host "  OpenAI API key (sk-...)"
        if (-not [string]::IsNullOrWhiteSpace($openaiKey)) {
            $env:OPENAI_API_KEY = $openaiKey
            Write-Ok "OPENAI_API_KEY set"
        }

        $anthropicKey = Read-Host "  Anthropic API key"
        if (-not [string]::IsNullOrWhiteSpace($anthropicKey)) {
            $env:ANTHROPIC_API_KEY = $anthropicKey
            Write-Ok "ANTHROPIC_API_KEY set"
        }

        $deepseekKey = Read-Host "  DeepSeek API key"
        if (-not [string]::IsNullOrWhiteSpace($deepseekKey)) {
            $env:DEEPSEEK_API_KEY = $deepseekKey
            Write-Ok "DEEPSEEK_API_KEY set"
        }

        $opencodeKey = Read-Host "  OpenCode Go API key"
        if (-not [string]::IsNullOrWhiteSpace($opencodeKey)) {
            $env:OPENCODE_GO_API_KEY = $opencodeKey
            Write-Ok "OPENCODE_GO_API_KEY set"
        }
    }

    Write-Host ""
}

# ── Build modules ────────────────────────────────────────────────────────────
if (-not $SkipBuild) {
    Write-Step "Checking if rebuild is needed ($BuildLabel mode)..."

    $cacheDir = Join-Path $PSScriptRoot ".build-cache"
    $modules = @(
        @{ name = "wireframe-ai-interface"; path = "kernel/interface" },
        @{ name = "wireframe-ai-context-core"; path = "modules/context-core" },
        @{ name = "wireframe-ai-orchestrator-core"; path = "modules/orchestrator-core" },
        @{ name = "wireframe-ai-sandbox-core"; path = "modules/sandbox-core" },
        @{ name = "wireframe-ai-event-sourcing-core"; path = "modules/event-sourcing-core" },
        @{ name = "wireframe-ai-integrations-core"; path = "modules/integrations-core" },
        @{ name = "wireframe-ai-observability-core"; path = "modules/observability-core" },
        @{ name = "wireframe-ai-provider-router-core"; path = "modules/provider-router-core" },
        @{ name = "wireframe-ai-tenant-core"; path = "modules/tenant-core" },
        @{ name = "wireframe-ai-webhooks-core"; path = "modules/webhooks-core" },
        @{ name = "wireframe-ai-interface-core"; path = "modules/interface-core" },
        @{ name = "adapter-rust"; path = "adapter/rust" },
        @{ name = "config"; path = "config" },
        @{ name = "tui-main"; path = "tools/tui-minimal" },
        @{ name = "agentic-sdk"; path = "sdk/agentic-sdk" }
    )

    $needsRebuild = $false
    $changedModules = @()

    foreach ($module in $modules) {
        $modulePath = Join-Path $RootDir $module.path
        $cacheFile = Join-Path $cacheDir "$($module.name).hash"

        if (-not (Test-Path $modulePath)) {
            continue
        }

        $currentHash = Get-PackageSourceHash -PackagePath $modulePath

        if ((Test-Path $cacheFile)) {
            $cachedHash = Get-Content $cacheFile -Raw
            if ($cachedHash -ne $currentHash) {
                $needsRebuild = $true
                $changedModules += $module.name
            }
        } else {
            $needsRebuild = $true
            $changedModules += $module.name
        }
    }

    if ($needsRebuild) {
        Write-InfoMsg "Code changed in: $($changedModules -join ', ')"
        Write-Step "Building modules ($BuildLabel mode)..."
        Write-Muted "(this may take a while)"

        $buildTimer = Start-Timer

        # Build all workspace members
        Push-Location $RootDir
        try {
            if ($BuildMode -eq "release") {
                cargo build --release
            } else {
                cargo build
            }
            if ($LASTEXITCODE -ne 0) {
                throw "Build failed (exit code: $LASTEXITCODE)"
            }
        } finally {
            Pop-Location
        }

        # Update cache
        if (-not (Test-Path $cacheDir)) {
            New-Item -ItemType Directory -Path $cacheDir -Force | Out-Null
        }

        foreach ($module in $modules) {
            $modulePath = Join-Path $RootDir $module.path
            $cacheFile = Join-Path $cacheDir "$($module.name).hash"
            if (Test-Path $modulePath) {
                $hash = Get-PackageSourceHash -PackagePath $modulePath
                if ($hash) {
                    $hash | Out-File $cacheFile
                }
            }
        }

        $buildTime = Stop-Timer -Timer $buildTimer
        $durationStr = Format-Duration -Duration $buildTime
        Write-Ok "Build finished in $durationStr"
    } else {
        Write-Ok "No code changes detected - skipping build"
    }
} else {
    Write-Step "Skipping build (--SkipBuild)"
}

# ── Start NATS ──────────────────────────────────────────────────────────────
Write-Step "Starting NATS message bus..."

$natsAlready = Test-ProcessRunning -ProcessName "nats-server"
if ($natsAlready) {
    Write-Ok "NATS already running"
} elseif (Test-Path $NatsBin) {
    try {
        $natsProc = Start-TrackedProcess -Name "nats-server" -Path $NatsBin -Hidden
        if ($natsProc) {
            Write-Ok "NATS started (PID $($natsProc.Id))"
        }
    } catch {
        Write-Warn "Failed to start NATS: $($_.Exception.Message)"
    }
} else {
    Write-Warn "NATS binary not available - start NATS manually"
}

Start-Sleep -Seconds 2

# ── Start modules ────────────────────────────────────────────────────────────
Write-Step "Starting modules..."

$cargoRun = if ($BuildMode -eq "release") {
    "cargo run --release -p"
} else {
    "cargo run -p"
}

if ($psmuxAvailable) {
    # Use psmux to create panes in a separate window
    Write-InfoMsg "Creating psmux session in separate window..."

    try {
        # Kill any existing wireframe-ai session
        & $psmuxCmdName kill-session -t wireframe-ai 2>$null

        # Create a new session with the first pane (context module)
        $ctxCmd = "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-context-core"
        & $psmuxCmdName new-session -d -s wireframe-ai -n Context "powershell" -c $ctxCmd
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create psmux session"
        }
        Write-Ok "Context module pane created"

        # Orchestrator pane (optional)
        if (-not $SkipOrchestrator) {
            Write-InfoMsg "Starting orchestrator pane..."
            $orchCmd = "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-orchestrator-core"
            & $psmuxCmdName new-window -d -t wireframe-ai -n Orchestrator "powershell" -c $orchCmd
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to create orchestrator pane"
            }
            Write-Ok "Orchestrator pane created"
        } else {
            Write-Ok "Orchestrator skipped (--SkipOrchestrator)"
        }

        # Sandbox pane
        Write-InfoMsg "Starting sandbox pane..."
        $sandboxCmd = "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-sandbox-core"
        & $psmuxCmdName new-window -d -t wireframe-ai -n Sandbox "powershell" -c $sandboxCmd
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to create sandbox pane"
        }
        Write-Ok "Sandbox pane created"

        # Python adapter pane (optional)
        if (-not $SkipAdapter) {
            Write-InfoMsg "Starting Python adapter pane..."
            $sdkDir = Join-Path (Join-Path $RootDir "sdk") "agentic-sdk-py\src"
            $adapterDir = Join-Path (Join-Path $RootDir "adapter") "python\src"
            $pythonPath = "$sdkDir;$adapterDir"
            $adapterCmd = "`$env:PYTHONPATH='$pythonPath'; cd '$RootDir'; python -m adapter"
            & $psmuxCmdName new-window -d -t wireframe-ai -n Adapter "powershell" -c $adapterCmd
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to create adapter pane"
            }
            Write-Ok "Python adapter pane created"
        } else {
            Write-Ok "Python adapter skipped (--SkipAdapter)"
        }

        # Open psmux in a new window (detached from current terminal)
        Write-InfoMsg "Opening psmux session in new window..."
        Start-Process $psmuxCmdName -ArgumentList "attach-session", "-t", "wireframe-ai"
        Write-Ok "psmux session opened in new window"

        # Don't start modules separately since they're in psmux panes
        $skipModuleStart = $true

    } catch {
        Write-Warn "Failed to create psmux session: $($_.Exception.Message)"
        Write-Warn "Falling back to separate windows..."
        $psmuxAvailable = $false
        $skipModuleStart = $false
    }
}

if (-not $skipModuleStart) {
    # Fallback: Start modules in separate windows
    # Context module
    Write-InfoMsg "Starting context module..."
    try {
        $ctx = Start-TrackedProcess -Name "context" -Path "powershell.exe" -Arguments @("-NoExit", "-Command", "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-context-core") -WorkingDirectory $RootDir
        if ($ctx) {
            Write-Ok "Context module started (PID $($ctx.Id))"
        }
    } catch {
        Write-Warn "Failed to start context module: $($_.Exception.Message)"
    }
    Start-Sleep -Seconds 1

    # Orchestrator (optional)
    if (-not $SkipOrchestrator) {
        Write-InfoMsg "Starting orchestrator..."
        try {
            $orch = Start-TrackedProcess -Name "orchestrator" -Path "powershell.exe" -Arguments @("-NoExit", "-Command", "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-orchestrator-core") -WorkingDirectory $RootDir
            if ($orch) {
                Write-Ok "Orchestrator started (PID $($orch.Id))"
            }
        } catch {
            Write-Warn "Failed to start orchestrator: $($_.Exception.Message)"
        }
        Start-Sleep -Seconds 1
    } else {
        Write-Ok "Orchestrator skipped (--SkipOrchestrator)"
    }

    # Sandbox
    Write-InfoMsg "Starting sandbox..."
    try {
        $sandbox = Start-TrackedProcess -Name "sandbox" -Path "powershell.exe" -Arguments @("-NoExit", "-Command", "cd '$RootDir'; cargo run $BuildFlag -p wireframe-ai-sandbox-core") -WorkingDirectory $RootDir
        if ($sandbox) {
            Write-Ok "Sandbox started (PID $($sandbox.Id))"
        }
    } catch {
        Write-Warn "Failed to start sandbox: $($_.Exception.Message)"
    }
    Start-Sleep -Seconds 1

    # Python adapter (optional) — via module
    if (-not $SkipAdapter) {
        Write-InfoMsg "Starting Python adapter..."
        try {
            $sdkDir = Join-Path (Join-Path $RootDir "sdk") "agentic-sdk-py\src"
            $adapterDir = Join-Path (Join-Path $RootDir "adapter") "python\src"
            $pythonPath = "$sdkDir;$adapterDir"
            $adapter = Start-TrackedProcess -Name "adapter" -Path "powershell.exe" -Arguments @("-NoExit", "-Command", "`$env:PYTHONPATH='$pythonPath'; cd '$RootDir'; python -m adapter") -WorkingDirectory $RootDir
            if ($adapter) {
                Write-Ok "Python adapter started (PID $($adapter.Id))"
            }
        } catch {
            Write-Warn "Failed to start Python adapter: $($_.Exception.Message)"
        }
        Start-Sleep -Seconds 1
    } else {
        Write-Ok "Python adapter skipped (--SkipAdapter)"
    }
}

# ── Summary ──────────────────────────────────────────────────────────────────
Write-Step "Launch summary"
Write-Host ""
if ($psmuxAvailable) {
    Write-Host "  All modules started in psmux panes (single window)." -ForegroundColor $global:ScriptColors.Label
    Write-Host "  Use psmux keybindings to navigate between panes." -ForegroundColor $global:ScriptColors.Label
} else {
    Write-Host "  All modules started in separate windows." -ForegroundColor $global:ScriptColors.Label
    Write-Host "  Check each window for module status." -ForegroundColor $global:ScriptColors.Label
}
Write-Host ""
Write-Host "  -----------------------------------------------------" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

# TEAM_015: Register cleanup handler to stop processes on script exit
# This ensures modules are closed when main terminal stops (Ctrl+C, window close, etc.)
Register-Cleanup -Handler {
    Write-Host ""
    Write-Warn "Cleaning up processes..."
    Stop-AllTrackedProcesses
}

# ── Open the TUI ──────────────────────────────────────────────────────────────
Write-Step "Opening TUI"
Write-Host ""
Write-Host "  Minimal TUI - Simple, fast interface for Wireframe-AI" -ForegroundColor $global:ScriptColors.Label
Write-Host ""
Write-Host "  Keyboard shortcuts:" -ForegroundColor $global:ScriptColors.Info
Write-Muted "    Ctrl+C or Ctrl+Q - Quit"
Write-Muted "    Enter - Submit message"
Write-Muted "    Backspace - Delete character"
Write-Muted "    Character keys - Type input"
Write-Host ""
Write-Host "  Configuration:" -ForegroundColor $global:ScriptColors.Info
Write-Muted "    Create tui-config.toml in working directory"
Write-Muted "    Configure NATS URL and provider settings"
Write-Muted "    See tools/tui-minimal/README.md for details"
Write-Host ""

# Set RUST_LOG for debug output
$env:RUST_LOG = "debug"

# Run the TUI in the current window so the user can interact
Push-Location $TuiPath
try {
    # Run minimal TUI
    if ($BuildMode -eq "release") {
        cargo run --release --bin tui-minimal
    } else {
        cargo run --bin tui-minimal
    }
} finally {
    Pop-Location
}

# ── Cleanup ──────────────────────────────────────────────────────────────────
Write-Step "Shutting down..."

# TEAM_015: Ensure all tracked processes are stopped
Stop-AllTrackedProcesses

Write-Host ""
Write-Ok "All modules stopped. Goodbye!"
Write-Host ""
