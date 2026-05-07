# Incident/Alert Hook Configuration for Wireframe-AI
# Sets up autonomous agents to trigger automatically in response to specific events

param(
    [string]$EventType = "",
    [string]$WebhookUrl = "",
    [switch]$ListHooks,
    [switch]$RemoveHook
)

Write-Host "🚨 Wireframe-AI Incident/Alert Hook Configuration" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

$hooksConfigFile = ".devin/incident-hooks.json"

# Initialize hooks config if it doesn't exist
if (-not (Test-Path $hooksConfigFile)) {
    $initialConfig = @{
        hooks = @()
    }
    $initialConfig | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8
}

# List existing hooks
if ($ListHooks) {
    Write-Host "`n📋 Configured Incident Hooks:" -ForegroundColor Yellow
    
    $config = Get-Content $hooksConfigFile | ConvertFrom-Json
    
    if ($config.hooks.Count -eq 0) {
        Write-Host "No hooks configured" -ForegroundColor Gray
    } else {
        foreach ($hook in $config.hooks) {
            Write-Host "`nHook ID: $($hook.id)" -ForegroundColor Green
            Write-Host "  Event Type: $($hook.eventType)" -ForegroundColor Gray
            Write-Host "  Webhook URL: $($hook.webhookUrl)" -ForegroundColor Gray
            Write-Host "  Agent Command: $($hook.agentCommand)" -ForegroundColor Gray
            Write-Host "  Created: $($hook.created)" -ForegroundColor Gray
        }
    }
    
    exit 0
}

# Remove a hook
if ($RemoveHook -and $EventType) {
    Write-Host "`n🗑️ Removing hook for event type: $EventType" -ForegroundColor Yellow
    
    $config = Get-Content $hooksConfigFile | ConvertFrom-Json
    $config.hooks = $config.hooks | Where-Object { $_.eventType -ne $EventType }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8
    Write-Host "✓ Hook removed" -ForegroundColor Green
    
    exit 0
}

# Add a new hook
if ([string]::IsNullOrEmpty($EventType) -or [string]::IsNullOrEmpty($WebhookUrl)) {
    Write-Host "`n⚠ Event type and webhook URL are required" -ForegroundColor Yellow
    Write-Host "Usage:" -ForegroundColor Gray
    Write-Host "  -ListHooks: List all configured hooks" -ForegroundColor Gray
    Write-Host "  -EventType <type> -WebhookUrl <url>: Add new hook" -ForegroundColor Gray
    Write-Host "  -RemoveHook -EventType <type>: Remove hook" -ForegroundColor Gray
    Write-Host "`nSupported event types:" -ForegroundColor Yellow
    Write-Host "  - build_failure" -ForegroundColor Gray
    Write-Host "  - test_failure" -ForegroundColor Gray
    Write-Host "  -deployment_failure" -ForegroundColor Gray
    Write-Host "  -security_alert" -ForegroundColor Gray
    Write-Host "  -performance_alert" -ForegroundColor Gray
    Write-Host "  -nats_error" -ForegroundColor Gray
    Write-Host "  -database_error" -ForegroundColor Gray
    exit 1
}

Write-Host "`n📋 Adding hook for event type: $EventType" -ForegroundColor Yellow

# Determine agent command based on event type
$agentCommand = switch ($EventType) {
    "build_failure" { "powershell -ExecutionPolicy Bypass -File .devin/scripts/playbook-code-review.ps1" }
    "test_failure" { "powershell -ExecutionPolicy Bypass -File .devin/scripts/first-failing-test.ps1" }
    "deployment_failure" { "powershell -ExecutionPolicy Bypass -File .devin/scripts/validate-environment.ps1" }
    "security_alert" { "Invoke agent security-auditor to review recent changes" }
    "performance_alert" { "Invoke agent performance-optimizer to analyze bottlenecks" }
    "nats_error" { "Check NATS connection and message flow" }
    "database_error" { "Check database connectivity and schema integrity" }
    default { "Run general investigation" }
}

# Create hook configuration
$hookId = [guid]::NewGuid().ToString()
$hook = @{
    id = $hookId
    eventType = $EventType
    webhookUrl = $WebhookUrl
    agentCommand = $agentCommand
    created = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    enabled = $true
}

# Add to configuration
$config = Get-Content $hooksConfigFile | ConvertFrom-Json
$config.hooks += $hook

$config | ConvertTo-Json -Depth 10 | Out-File $hooksConfigFile -Encoding UTF8

Write-Host "✓ Hook added successfully" -ForegroundColor Green
Write-Host "  Hook ID: $hookId" -ForegroundColor Gray
Write-Host "  Event Type: $EventType" -ForegroundColor Gray
Write-Host "  Agent Command: $agentCommand" -ForegroundColor Gray

exit 0
