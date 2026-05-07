# Configure MCP Servers for Wireframe-AI Autonomous Mode
# Sets up MCP server integration for external tools and data access

param(
    [switch]$EnableAll,
    [switch]$DisableAll,
    [switch]$ListCurrent,
    [switch]$AddGitHub,
    [switch]$AddChromeDevTools,
    [switch]$TestMode
)

$mcpConfigFile = ".devin/mcp-config.json"
$autonomousConfigFile = ".devin/autonomous-mode.json"

Write-Host "🔌 Wireframe-AI MCP Server Configuration" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

# List current MCP servers
if ($ListCurrent) {
    Write-Host "`n📋 Current MCP Servers:" -ForegroundColor Yellow
    
    if (Test-Path $mcpConfigFile) {
        $config = Get-Content $mcpConfigFile | ConvertFrom-Json
        
        if ($config.mcpServers.Count -eq 0) {
            Write-Host "No MCP servers configured" -ForegroundColor Gray
        } else {
            foreach ($serverName in $config.mcpServers.PSObject.Properties.Name) {
                $server = $config.mcpServers.$serverName
                $status = if ($server.enabled) { "✓ Enabled" } else { "✗ Disabled" }
                Write-Host "`nServer: $serverName" -ForegroundColor Green
                Write-Host "  Command: $($server.command)" -ForegroundColor Gray
                Write-Host "  Status: $status" -ForegroundColor $(if ($server.enabled) { "Green" } else { "Red" })
                if ($server.env) {
                    Write-Host "  Environment Variables: $($server.env.PSObject.Properties.Count)" -ForegroundColor Gray
                }
            }
        }
    } else {
        Write-Host "No MCP configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Disable all MCP servers
if ($DisableAll) {
    Write-Host "`n🗑️ Disabling all MCP servers..." -ForegroundColor Yellow
    
    if (Test-Path $mcpConfigFile) {
        $config = Get-Content $mcpConfigFile | ConvertFrom-Json
        foreach ($serverName in $config.mcpServers.PSObject.Properties.Name) {
            $config.mcpServers.$serverName.enabled = $false
        }
        $config | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
        Write-Host "✓ All MCP servers disabled" -ForegroundColor Green
    } else {
        Write-Host "No MCP configuration file found" -ForegroundColor Yellow
    }
    
    exit 0
}

# Enable all MCP servers from autonomous config
if ($EnableAll) {
    Write-Host "`n🚀 Enabling MCP servers from autonomous configuration..." -ForegroundColor Yellow
    
    if (-not (Test-Path $autonomousConfigFile)) {
        Write-Host "✗ Autonomous mode configuration not found" -ForegroundColor Red
        exit 1
    }
    
    $autonomousConfig = Get-Content $autonomousConfigFile | ConvertFrom-Json
    
    # Initialize MCP config if it doesn't exist
    if (-not (Test-Path $mcpConfigFile)) {
        $initialConfig = @{
            mcpServers = @{}
        }
        $initialConfig | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    }
    
    $config = Get-Content $mcpConfigFile | ConvertFrom-Json
    
    # Add MCP servers from autonomous config
    foreach ($serverName in $autonomousConfig.settings.mcpIntegration.servers.PSObject.Properties.Name) {
        $serverConfig = $autonomousConfig.settings.mcpIntegration.servers.$serverName
        
        if ($serverConfig.enabled) {
            $mcpServerConfig = Get-MCPServerConfig -ServerName $serverName -Config $serverConfig
            
            if ($config.mcpServers.PSObject.Properties.Name -contains $serverName) {
                $config.mcpServers.$serverName.enabled = $true
                Write-Host "  Updated existing server: $serverName" -ForegroundColor Yellow
            } else {
                $config.mcpServers | Add-Member -NotePropertyName $serverName -NotePropertyValue $mcpServerConfig
                Write-Host "  Added new server: $serverName" -ForegroundColor Green
            }
        }
    }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    Write-Host "`n✓ MCP servers configured successfully" -ForegroundColor Green
    Write-Host "  Total servers: $($config.mcpServers.PSObject.Properties.Count)" -ForegroundColor Gray
    
    exit 0
}

# Add GitHub MCP server
if ($AddGitHub) {
    Write-Host "`n➕ Adding GitHub MCP server..." -ForegroundColor Yellow
    
    if (-not (Test-Path $mcpConfigFile)) {
        $initialConfig = @{
            mcpServers = @{}
        }
        $initialConfig | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    }
    
    $config = Get-Content $mcpConfigFile | ConvertFrom-Json
    
    $githubServer = @{
        command = "npx"
        args = @("-y", "@modelcontextprotocol/server-github")
        env = @{
            GITHUB_TOKEN = "${GITHUB_TOKEN}"
        }
        enabled = $true
    }
    
    if ($config.mcpServers.PSObject.Properties.Name -contains "github") {
        $config.mcpServers.github = $githubServer
        Write-Host "  Updated GitHub server" -ForegroundColor Yellow
    } else {
        $config.mcpServers | Add-Member -NotePropertyName "github" -NotePropertyValue $githubServer
        Write-Host "  Added GitHub server" -ForegroundColor Green
    }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    Write-Host "✓ GitHub MCP server configured" -ForegroundColor Green
    Write-Host "⚠ Remember to set GITHUB_TOKEN environment variable" -ForegroundColor Yellow
    
    exit 0
}

# Add Chrome DevTools MCP server
if ($AddChromeDevTools) {
    Write-Host "`n➕ Adding Chrome DevTools MCP server..." -ForegroundColor Yellow
    
    if (-not (Test-Path $mcpConfigFile)) {
        $initialConfig = @{
            mcpServers = @{}
        }
        $initialConfig | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    }
    
    $config = Get-Content $mcpConfigFile | ConvertFrom-Json
    
    $chromeServer = @{
        command = "npx"
        args = @("-y", "@chrome-devtools/mcp-server")
        env = @{
            HEADLESS = "true"
        }
        enabled = $true
    }
    
    if ($config.mcpServers.PSObject.Properties.Name -contains "chrome-devtools") {
        $config.mcpServers."chrome-devtools" = $chromeServer
        Write-Host "  Updated Chrome DevTools server" -ForegroundColor Yellow
    } else {
        $config.mcpServers | Add-Member -NotePropertyName "chrome-devtools" -NotePropertyValue $chromeServer
        Write-Host "  Added Chrome DevTools server" -ForegroundColor Green
    }
    
    $config | ConvertTo-Json -Depth 10 | Out-File $mcpConfigFile -Encoding UTF8
    Write-Host "✓ Chrome DevTools MCP server configured" -ForegroundColor Green
    
    exit 0
}

# Default: show usage
Write-Host "`nUsage:" -ForegroundColor Yellow
Write-Host "  -EnableAll: Enable all MCP servers from autonomous-mode.json" -ForegroundColor Gray
Write-Host "  -DisableAll: Disable all MCP servers" -ForegroundColor Gray
Write-Host "  -ListCurrent: List current MCP servers" -ForegroundColor Gray
Write-Host "  -AddGitHub: Add GitHub MCP server" -ForegroundColor Gray
Write-Host "  -AddChromeDevTools: Add Chrome DevTools MCP server" -ForegroundColor Gray
Write-Host "  -TestMode: Test MCP configuration without applying" -ForegroundColor Gray

function Get-MCPServerConfig {
    param([string]$ServerName, [object]$Config)
    
    $configs = @{
        "filesystem" = @{
            command = "npx"
            args = @("-y", "@modelcontextprotocol/server-filesystem", "C:\Users\Takon\Development\Projects\Wireframe-AI")
            enabled = $true
        }
        "git" = @{
            command = "npx"
            args = @("-y", "@modelcontextprotocol/server-git", "--repository", "C:\Users\Takon\Development\Projects\Wireframe-AI")
            enabled = $true
        }
        "fetch" = @{
            command = "npx"
            args = @("-y", "@modelcontextprotocol/server-fetch")
            enabled = $true
        }
        "memory" = @{
            command = "npx"
            args = @("-y", "@modelcontextprotocol/server-memory")
            enabled = $true
        }
    }
    
    return $configs[$ServerName]
}
