# Configure Playbook Automation for Wireframe-AI
# Sets up automated playbook execution for hands-off operation

param(
    [switch]$EnableAll,
    [switch]$DisableAll,
    [switch]$ListCurrent,
    [switch]$TestMode,
    [string]$RunPlaybook
)

$playbookConfigFile = ".devin/playbook-automation.json"
$autonomousConfigFile = ".devin/autonomous-mode.json"

Write-Host "📚 Wireframe-AI Playbook Automation Configuration" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# List current playbook automation
if ($ListCurrent) {
    Write-Host "`n📋 Current Playbook Automation:" -ForegroundColor Yellow
    
    if (Test-Path $playbookConfigFile) {
        $config = Get-Content $playbookConfigFile | ConvertFrom-Json
        
        if ($config.playbooks.Count -eq 0) {
            Write-Host "No playbook automation configured" -ForegroundColor Gray
        } else {
            foreach ($playbookName in $config.playbooks.PSObject.Properties.Name) {
                $playbook = $config.playbooks.$playbookName
                $status = if ($playbook.enabled) { "✓ Enabled" } else { "✗ Disabled" }
                Write-Host "`nPlaybook: $playbookName" -ForegroundColor Green
                Write-Host "  Script: $($playbook.script)" -ForegroundColor Gray
                Write-Host "  Status: $status" -ForegroundColor $(if ($playbook.enabled) { "Green" } else { "Red" })
                if ($playbook.schedule) {
                    Write-Host "  Schedule: $($playbook.schedule)" -ForegroundColor Gray
                }
                if ($playbook.trigger) {
                    Write-Host "  Trigger: $($playbook.trigger)" -ForegroundColor Gray
                }
                Write-Host "  Auto-approve: $($playbook.autoApprove)" -ForegroundColor Gray
            }
        }
    } else {
        Write-Host "No playbook automation configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Disable all playbook automation
if ($DisableAll) {
    Write-Host "`n🗑️ Disabling all playbook automation..." -ForegroundColor Yellow
    
    if (Test-Path $playbookConfigFile) {
        $config = Get-Content $playbookConfigFile | ConvertFrom-Json
        foreach ($playbookName in $config.playbooks.PSObject.Properties.Name) {
            $config.playbooks.$playbookName.enabled = $false
        }
        $config | ConvertTo-Json -Depth 10 | Out-File $playbookConfigFile -Encoding UTF8
        Write-Host "✓ All playbook automation disabled" -ForegroundColor Green
    } else {
        Write-Host "No playbook automation configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Enable all playbooks from autonomous config
if ($EnableAll) {
    Write-Host "`n🚀 Enabling playbook automation from configuration..." -ForegroundColor Yellow
    
    if (-not (Test-Path $autonomousConfigFile)) {
        Write-Host "✗ Autonomous mode configuration not found" -ForegroundColor Red
        exit 1
    }
    
    $autonomousConfig = Get-Content $autonomousConfigFile | ConvertFrom-Json
    
    # Initialize playbook config if it doesn't exist
    if (-not (Test-Path $playbookConfigFile)) {
        $initialConfig = @{
            playbooks = @{}
            lastRun = @{}
        }
        $initialConfig | ConvertTo-Json -Depth 10 | Out-File $playbookConfigFile -Encoding UTF8
    }
    
    $config = Get-Content $playbookConfigFile | ConvertFrom-Json
    
    # Add playbooks from autonomous config
    foreach ($playbookName in $autonomousConfig.settings.playbookAutomation.playbooks.PSObject.Properties.Name) {
        $playbookConfig = $autonomousConfig.settings.playbookAutomation.playbooks.$playbookName
        
        if ($playbookConfig.enabled) {
            $playbookDef = Get-PlaybookDefinition -PlaybookName $playbookName -Config $playbookConfig
            
            if ($config.playbooks.PSObject.Properties.Name -contains $playbookName) {
                $config.playbooks.$playbookName.enabled = $true
                Write-Host "  Updated existing playbook: $playbookName" -ForegroundColor Yellow
            } else {
                $config.playbooks | Add-Member -NotePropertyName $playbookName -NotePropertyValue $playbookDef
                Write-Host "  Added new playbook: $playbookName" -ForegroundColor Green
            }
        }
    }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $playbookConfigFile -Encoding UTF8
    Write-Host "`n✓ Playbook automation configured successfully" -ForegroundColor Green
    Write-Host "  Total playbooks: $($config.playbooks.PSObject.Properties.Count)" -ForegroundColor Gray
    
    exit 0
}

# Run a specific playbook
if ($RunPlaybook) {
    Write-Host "`n▶️ Running playbook: $RunPlaybook" -ForegroundColor Yellow
    
    if (-not (Test-Path $playbookConfigFile)) {
        Write-Host "✗ Playbook automation configuration not found" -ForegroundColor Red
        exit 1
    }
    
    $config = Get-Content $playbookConfigFile | ConvertFrom-Json
    
    if (-not $config.playbooks.PSObject.Properties.Name -contains $RunPlaybook) {
        Write-Host "✗ Playbook '$RunPlaybook' not found" -ForegroundColor Red
        exit 1
    }
    
    $playbook = $config.playbooks.$RunPlaybook
    
    if (-not $playbook.enabled) {
        Write-Host "✗ Playbook '$RunPlaybook' is disabled" -ForegroundColor Red
        exit 1
    }
    
    Write-Host "  Script: $($playbook.script)" -ForegroundColor Gray
    Write-Host "  Auto-approve: $($playbook.autoApprove)" -ForegroundColor Gray
    
    if (-not $playbook.autoApprove -and -not $TestMode) {
        $response = Read-Host "Approve execution? (y/N)"
        if ($response -ne "y") {
            Write-Host "✗ Playbook execution cancelled" -ForegroundColor Yellow
            exit 0
        }
    }
    
    # Run the playbook
    $scriptPath = $playbook.script
    if (Test-Path $scriptPath) {
        Write-Host "  Executing..." -ForegroundColor Green
        $output = & powershell -ExecutionPolicy Bypass -File $scriptPath
        $exitCode = $LASTEXITCODE
        
        # Update last run time
        $config.lastRun.$RunPlaybook = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
        $config | ConvertTo-Json -Depth 10 | Out-File $playbookConfigFile -Encoding UTF8
        
        if ($exitCode -eq 0) {
            Write-Host "✓ Playbook completed successfully" -ForegroundColor Green
        } else {
            Write-Host "✗ Playbook failed with exit code: $exitCode" -ForegroundColor Red
        }
    } else {
        Write-Host "✗ Script not found: $scriptPath" -ForegroundColor Red
        exit 1
    }
    
    exit 0
}

# Default: show usage
Write-Host "`nUsage:" -ForegroundColor Yellow
Write-Host "  -EnableAll: Enable all playbooks from autonomous-mode.json" -ForegroundColor Gray
Write-Host "  -DisableAll: Disable all playbook automation" -ForegroundColor Gray
Write-Host "  -ListCurrent: List current playbook automation" -ForegroundColor Gray
Write-Host "  -RunPlaybook <name>: Run a specific playbook" -ForegroundColor Gray
Write-Host "  -TestMode: Test playbook configuration without applying" -ForegroundColor Gray

function Get-PlaybookDefinition {
    param([string]$PlaybookName, [object]$Config)
    
    $scripts = @{
        "dependency_upgrade" = ".devin/scripts/playbook-dependency-upgrade.ps1"
        "code_review" = ".devin/scripts/playbook-code-review.ps1"
        "feature_flag_removal" = ".devin/scripts/playbook-feature-flag-removal.ps1"
        "test_addition" = ".devin/scripts/playbook-test-addition.ps1"
    }
    
    $playbookDef = @{
        script = $scripts[$PlaybookName]
        enabled = $true
        autoApprove = $Config.autoApprove
        notification = $Config.notification
    }
    
    if ($Config.schedule) {
        $playbookDef.schedule = "$($Config.day) at $($Config.time)"
    }
    
    if ($Config.trigger) {
        $playbookDef.trigger = $Config.trigger
    }
    
    return $playbookDef
}
