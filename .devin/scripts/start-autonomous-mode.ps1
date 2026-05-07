# Start Wireframe-AI in Autonomous Mode
# Self-running system with minimal human oversight

param(
    [switch]$Interactive,
    [switch]$DryRun,
    [switch]$Monitor,
    [int]$MaxIterations = 15
)

$autonomousConfigFile = ".devin/autonomous-mode.json"
$killswitchFile = "$env:USERPROFILE\.wireframe-ai-autonomous-stop"
$logDir = ".devin/logs/autonomous"

Write-Host "🤖 Wireframe-AI Autonomous Mode" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# Check if autonomous mode is enabled
if (-not (Test-Path $autonomousConfigFile)) {
    Write-Host "✗ Autonomous mode configuration not found" -ForegroundColor Red
    Write-Host "Run configure scripts first to set up autonomous mode" -ForegroundColor Yellow
    exit 1
}

$config = Get-Content $autonomousConfigFile | ConvertFrom-Json

if (-not $config.enabled) {
    Write-Host "✗ Autonomous mode is disabled in configuration" -ForegroundColor Red
    exit 1
}

Write-Host "✓ Autonomous mode enabled" -ForegroundColor Green

# Create log directory
New-Item -ItemType Directory -Force -Path $logDir | Out-Null
$logFile = "$logDir/autonomous-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"

Write-Host "📋 Configuration:" -ForegroundColor Yellow
Write-Host "  Agent Triggering: $($config.settings.agentTriggering.enabled)" -ForegroundColor Gray
Write-Host "  MCP Integration: $($config.settings.mcpIntegration.enabled)" -ForegroundColor Gray
Write-Host "  Playbook Automation: $($config.settings.playbookAutomation.enabled)" -ForegroundColor Gray
Write-Host "  Self-Running System: $($config.settings.selfRunningSystem.enabled)" -ForegroundColor Gray
Write-Host "  Log File: $logFile" -ForegroundColor Gray

# Check killswitch
if (Test-Path $killswitchFile) {
    Write-Host "⚠ Killswitch file exists. Removing it..." -ForegroundColor Yellow
    Remove-Item $killswitchFile
}

# Monitor mode
if ($Monitor) {
    Write-Host "`n👀 Monitoring autonomous mode (Ctrl+C to stop)..." -ForegroundColor Yellow
    
    while ($true) {
        if (Test-Path $killswitchFile) {
            Write-Host "🛑 Killswitch tripped. Stopping autonomous mode." -ForegroundColor Red
            Remove-Item $killswitchFile
            break
        }
        
        # Check system health
        $health = Get-SystemHealth
        Write-Host "$(Get-Date -Format 'HH:mm:ss') - Health: $($health.status)" -ForegroundColor $(if ($health.status -eq "healthy") { "Green" } else { "Red" })
        
        if ($health.status -ne "healthy") {
            Write-Host "  Issues: $($health.issues -join ', ')" -ForegroundColor Yellow
        }
        
        Start-Sleep -Seconds 30
    }
    
    exit 0
}

# Interactive mode
if ($Interactive) {
    Write-Host "`n🎮 Interactive Autonomous Mode" -ForegroundColor Yellow
    Write-Host "Type 'help' for available commands" -ForegroundColor Gray
    
    while ($true) {
        $input = Read-Host "autonomous>"
        
        switch ($input.ToLower()) {
            "help" {
                Write-Host "Available commands:" -ForegroundColor Yellow
                Write-Host "  status - Show system status" -ForegroundColor Gray
                Write-Host "  build - Run build" -ForegroundColor Gray
                Write-Host "  test - Run tests" -ForegroundColor Gray
                Write-Host "  lint - Run linting" -ForegroundColor Gray
                Write-Host "  hooks - Show incident hooks" -ForegroundColor Gray
                Write-Host "  mcp - Show MCP servers" -ForegroundColor Gray
                Write-Host "  playbooks - Show playbook automation" -ForegroundColor Gray
                Write-Host "  exit - Exit interactive mode" -ForegroundColor Gray
            }
            "status" {
                Show-SystemStatus
            }
            "build" {
                Run-Build -DryRun $DryRun
            }
            "test" {
                Run-Tests -DryRun $DryRun
            }
            "lint" {
                Run-Lint -DryRun $DryRun
            }
            "hooks" {
                & powershell -ExecutionPolicy Bypass -File .devin/scripts/configure-autonomous-hooks.ps1 -ListCurrent
            }
            "mcp" {
                & powershell -ExecutionPolicy Bypass -File .devin/scripts/configure-mcp-servers.ps1 -ListCurrent
            }
            "playbooks" {
                & powershell -ExecutionPolicy Bypass -File .devin/scripts/configure-playbook-automation.ps1 -ListCurrent
            }
            "exit" {
                Write-Host "Exiting interactive mode" -ForegroundColor Yellow
                break
            }
            default {
                Write-Host "Unknown command: $input" -ForegroundColor Red
                Write-Host "Type 'help' for available commands" -ForegroundColor Gray
            }
        }
        
        if (Test-Path $killswitchFile) {
            Write-Host "🛑 Killswitch tripped. Exiting." -ForegroundColor Red
            Remove-Item $killswitchFile
            break
        }
    }
    
    exit 0
}

# Dry run mode
if ($DryRun) {
    Write-Host "`n🔍 Dry Run Mode - No actual changes will be made" -ForegroundColor Yellow
    Write-Host "Simulating autonomous operations..." -ForegroundColor Gray
    
    $behaviors = $config.settings.selfRunningSystem.behaviors
    Write-Host "`nSimulated Behaviors:" -ForegroundColor Yellow
    Write-Host "  Auto Build: $($behaviors.autoBuild)" -ForegroundColor Gray
    Write-Host "  Auto Test: $($behaviors.autoTest)" -ForegroundColor Gray
    Write-Host "  Auto Lint: $($behaviors.autoLint)" -ForegroundColor Gray
    Write-Host "  Auto Format: $($behaviors.autoFormat)" -ForegroundColor Gray
    Write-Host "  Auto Commit: $($behaviors.autoCommit)" -ForegroundColor Gray
    Write-Host "  Auto Push: $($behaviors.autoPush)" -ForegroundColor Gray
    
    Write-Host "`nDecision Points:" -ForegroundColor Yellow
    $decisions = $config.settings.selfRunningSystem.decisionPoints
    Write-Host "  Can Edit Files: $($decisions.canEditFiles)" -ForegroundColor Gray
    Write-Host "  Can Run Commands: $($decisions.canRunCommands)" -ForegroundColor Gray
    Write-Host "  Can Modify Tests: $($decisions.canModifyTests)" -ForegroundColor Gray
    Write-Host "  Can Change Dependencies: $($decisions.canChangeDependencies)" -ForegroundColor Gray
    Write-Host "  Can Modify Schema: $($decisions.canModifySchema)" -ForegroundColor Gray
    Write-Host "  Can Deploy: $($decisions.canDeploy)" -ForegroundColor Gray
    
    Write-Host "`nSafety Checks:" -ForegroundColor Yellow
    $safety = $config.settings.selfRunningSystem.safetyChecks
    Write-Host "  Require Build Pass: $($safety.requireBuildPass)" -ForegroundColor Gray
    Write-Host "  Require Tests Pass: $($safety.requireTestsPass)" -ForegroundColor Gray
    Write-Host "  Require Lint Pass: $($safety.requireLintPass)" -ForegroundColor Gray
    Write-Host "  Max Iterations: $($safety.maxIterations)" -ForegroundColor Gray
    Write-Host "  Killswitch: $killswitchFile" -ForegroundColor Gray
    
    Write-Host "`n✓ Dry run complete" -ForegroundColor Green
    exit 0
}

# Normal autonomous mode
Write-Host "`n🚀 Starting autonomous mode..." -ForegroundColor Yellow
Write-Host "Max iterations: $MaxIterations" -ForegroundColor Gray
Write-Host "Killswitch: $killswitchFile" -ForegroundColor Gray
Write-Host "Log file: $logFile" -ForegroundColor Gray

for ($i = 1; $i -le $MaxIterations; $i++) {
    Write-Host "`n── Iteration $i/$MaxIterations ──" -ForegroundColor Cyan
    
    # Check killswitch
    if (Test-Path $killswitchFile) {
        Write-Host "🛑 Killswitch tripped. Stopping autonomous mode." -ForegroundColor Red
        Remove-Item $killswitchFile
        break
    }
    
    # Log iteration start
    $logEntry = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') - Iteration $i started"
    Add-Content -Path $logFile -Value $logEntry
    
    # Run autonomous operations
    $behaviors = $config.settings.selfRunningSystem.behaviors
    $safety = $config.settings.selfRunningSystem.safetyChecks
    
    # Build
    if ($behaviors.autoBuild) {
        Write-Host "Running build..." -ForegroundColor Yellow
        $buildResult = Run-Build -DryRun $false
        if ($safety.requireBuildPass -and $buildResult -ne 0) {
            Write-Host "✗ Build failed, stopping iteration" -ForegroundColor Red
            continue
        }
    }
    
    # Test
    if ($behaviors.autoTest) {
        Write-Host "Running tests..." -ForegroundColor Yellow
        $testResult = Run-Tests -DryRun $false
        if ($safety.requireTestsPass -and $testResult -ne 0) {
            Write-Host "✗ Tests failed, stopping iteration" -ForegroundColor Red
            continue
        }
    }
    
    # Lint
    if ($behaviors.autoLint) {
        Write-Host "Running lint..." -ForegroundColor Yellow
        $lintResult = Run-Lint -DryRun $false
        if ($safety.requireLintPass -and $lintResult -ne 0) {
            Write-Host "✗ Lint failed, stopping iteration" -ForegroundColor Red
            continue
        }
    }
    
    # Check for human intervention requirements
    $decisions = $config.settings.selfRunningSystem.decisionPoints
    $humanIntervention = $config.settings.selfRunningSystem.humanIntervention
    
    $requiresIntervention = Check-HumanIntervention -Decisions $decisions -RequiredActions $humanIntervention.requiredFor
    
    if ($requiresIntervention) {
        Write-Host "⚠ Human intervention required for: $($requiresIntervention -join ', ')" -ForegroundColor Yellow
        Write-Host "Stopping iteration for human review" -ForegroundColor Yellow
        break
    }
    
    Write-Host "✓ Iteration $i completed successfully" -ForegroundColor Green
}

Write-Host "`n✓ Autonomous mode completed" -ForegroundColor Green
Write-Host "Log file: $logFile" -ForegroundColor Gray

function Show-SystemStatus {
    Write-Host "`n📊 System Status:" -ForegroundColor Yellow
    $health = Get-SystemHealth
    Write-Host "  Status: $($health.status)" -ForegroundColor $(if ($health.status -eq "healthy") { "Green" } else { "Red" })
    Write-Host "  Git Status: $(git status --porcelain | Measure-Object).Count files changed" -ForegroundColor Gray
    Write-Host "  Branch: $(git branch --show-current)" -ForegroundColor Gray
}

function Get-SystemHealth {
    $issues = @()
    
    # Check if git repo
    $gitStatus = git status 2>$null
    if ($LASTEXITCODE -ne 0) {
        $issues += "Not a git repository"
    }
    
    # Check if Cargo.toml exists
    if (-not (Test-Path "Cargo.toml")) {
        $issues += "Cargo.toml not found"
    }
    
    $status = if ($issues.Count -eq 0) { "healthy" } else { "unhealthy" }
    
    return @{
        status = $status
        issues = $issues
    }
}

function Run-Build {
    param([bool]$DryRun)
    
    if ($DryRun) {
        Write-Host "[DRY RUN] Would run: cargo build --release" -ForegroundColor Cyan
        return 0
    }
    
    $output = cargo build --release 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host "✓ Build succeeded" -ForegroundColor Green
    } else {
        Write-Host "✗ Build failed" -ForegroundColor Red
    }
    
    return $exitCode
}

function Run-Tests {
    param([bool]$DryRun)
    
    if ($DryRun) {
        Write-Host "[DRY RUN] Would run: cargo test" -ForegroundColor Cyan
        return 0
    }
    
    $output = cargo test 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host "✓ Tests passed" -ForegroundColor Green
    } else {
        Write-Host "✗ Tests failed" -ForegroundColor Red
    }
    
    return $exitCode
}

function Run-Lint {
    param([bool]$DryRun)
    
    if ($DryRun) {
        Write-Host "[DRY RUN] Would run: cargo clippy" -ForegroundColor Cyan
        return 0
    }
    
    $output = cargo clippy 2>&1
    $exitCode = $LASTEXITCODE
    
    if ($exitCode -eq 0) {
        Write-Host "✓ Lint passed" -ForegroundColor Green
    } else {
        Write-Host "✗ Lint failed" -ForegroundColor Red
    }
    
    return $exitCode
}

function Check-HumanIntervention {
    param([hashtable]$Decisions, [array]$RequiredActions)
    
    $requiresIntervention = @()
    
    # Check git status for changes that might require intervention
    $gitStatus = git status --porcelain 2>$null
    if ($gitStatus) {
        $changedFiles = $gitStatus | Measure-Object
        if ($changedFiles.Count -gt 0 -and -not $Decisions.canEditFiles) {
            $requiresIntervention += "file_changes"
        }
    }
    
    # Check for schema changes
    $schemaChanges = git diff --name-only | Where-Object { $_ -match "schemas/" }
    if ($schemaChanges -and -not $Decisions.canModifySchema) {
        $requiresIntervention += "schema_changes"
    }
    
    # Check for dependency changes
    $depChanges = git diff --name-only | Where-Object { $_ -match "Cargo.toml" }
    if ($depChanges -and -not $Decisions.canChangeDependencies) {
        $requiresIntervention += "dependency_changes"
    }
    
    return $requiresIntervention
}
