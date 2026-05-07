# Configure Autonomous Hooks for Wireframe-AI
# Sets up incident hooks for automatic agent triggering

param(
    [switch]$EnableAll,
    [switch]$DisableAll,
    [switch]$ListCurrent,
    [switch]$TestMode
)

$hooksConfigFile = ".devin/incident-hooks.json"
$autonomousConfigFile = ".devin/autonomous-mode.json"

Write-Host "🤖 Wireframe-AI Autonomous Hooks Configuration" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# List current hooks
if ($ListCurrent) {
    Write-Host "`n📋 Current Incident Hooks:" -ForegroundColor Yellow
    
    if (Test-Path $hooksConfigFile) {
        $config = Get-Content $hooksConfigFile | ConvertFrom-Json
        
        if ($config.hooks.Count -eq 0) {
            Write-Host "No hooks configured" -ForegroundColor Gray
        } else {
            foreach ($hook in $config.hooks) {
                $status = if ($hook.enabled) { "✓ Enabled" } else { "✗ Disabled" }
                Write-Host "`nHook ID: $($hook.id)" -ForegroundColor Green
                Write-Host "  Event Type: $($hook.eventType)" -ForegroundColor Gray
                Write-Host "  Agent Command: $($hook.agentCommand)" -ForegroundColor Gray
                Write-Host "  Status: $status" -ForegroundColor $(if ($hook.enabled) { "Green" } else { "Red" })
                Write-Host "  Created: $($hook.created)" -ForegroundColor Gray
            }
        }
    } else {
        Write-Host "No hooks configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Disable all hooks
if ($DisableAll) {
    Write-Host "`n🗑️ Disabling all incident hooks..." -ForegroundColor Yellow
    
    if (Test-Path $hooksConfigFile) {
        $config = Get-Content $hooksConfigFile | ConvertFrom-Json
        foreach ($hook in $config.hooks) {
            $hook.enabled = $false
        }
        $config | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8
        Write-Host "✓ All hooks disabled" -ForegroundColor Green
    } else {
        Write-Host "No hooks configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Enable all hooks from autonomous config
if ($EnableAll) {
    Write-Host "`n🚀 Enabling autonomous hooks from configuration..." -ForegroundColor Yellow
    
    if (-not (Test-Path $autonomousConfigFile)) {
        Write-Host "✗ Autonomous mode configuration not found" -ForegroundColor Red
        exit 1
    }
    
    $autonomousConfig = Get-Content $autonomousConfigFile | ConvertFrom-Json
    
    # Initialize hooks config if it doesn't exist
    if (-not (Test-Path $hooksConfigFile)) {
        $initialConfig = @{
            hooks = @()
        }
        $initialConfig | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8
    }
    
    $config = Get-Content $hooksConfigFile | ConvertFrom-Json
    
    # Add hooks from autonomous config
    foreach ($trigger in $autonomousConfig.settings.agentTriggering.triggers) {
        $hookId = [guid]::NewGuid().ToString()
        
        # Check if hook already exists
        $existingHook = $config.hooks | Where-Object { $_.eventType -eq $trigger.eventType }
        
        if ($existingHook) {
            $existingHook.enabled = $true
            $existingHook.agentCommand = Get-AgentCommand -Action $trigger.action
            Write-Host "  Updated existing hook: $($trigger.eventType)" -ForegroundColor Yellow
        } else {
            $hook = @{
                id = $hookId
                eventType = $trigger.eventType
                action = $trigger.action
                agent = $trigger.agent
                agentCommand = Get-AgentCommand -Action $trigger.action
                autoApprove = $trigger.autoApprove
                notification = $trigger.notification
                priority = if ($trigger.priority) { $trigger.priority } else { "normal" }
                enabled = $true
                created = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
            }
            $config.hooks += $hook
            Write-Host "  Added new hook: $($trigger.eventType)" -ForegroundColor Green
        }
    }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8
    Write-Host "`n✓ Autonomous hooks configured successfully" -ForegroundColor Green
    Write-Host "  Total hooks: $($config.hooks.Count)" -ForegroundColor Gray
    
    exit 0
}

# Default: show usage
Write-Host "`nUsage:" -ForegroundColor Yellow
Write-Host "  -EnableAll: Enable all hooks from autonomous-mode.json" -ForegroundColor Gray
Write-Host "  -DisableAll: Disable all hooks" -ForegroundColor Gray
Write-Host "  -ListCurrent: List current hooks" -ForegroundColor Gray
Write-Host "  -TestMode: Test hook configuration without applying" -ForegroundColor Gray

function Get-AgentCommand {
    param([string]$Action)
    
    $commands = @{
        "automated_code_review" = "powershell -ExecutionPolicy Bypass -File .devin/scripts/playbook-code-review.ps1"
        "first_failing_test" = "powershell -ExecutionPolicy Bypass -File .devin/scripts/first-failing-test.ps1"
        "environment_validation" = "powershell -ExecutionPolicy Bypass -File .devin/scripts/validate-environment.ps1"
        "security_audit" = "Invoke agent security-auditor for security review"
        "performance_optimization" = "Invoke agent performance-optimizer for analysis"
        "nats_diagnosis" = "Check NATS connection and message flow"
        "database_check" = "Check database connectivity and schema integrity"
    }
    
    return $commands[$Action]
}
