#!/usr/bin/env pwsh
# TEAM_006: Common utilities and functions for Wireframe-AI scripts
# Provides centralized error handling, logging, progress indicators, and retry logic

$ErrorActionPreference = "Stop"

# â”€â”€ Color Configuration â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
$global:ScriptColors = @{
    Info    = "Cyan"
    Ok      = "Green"
    Warn    = "Yellow"
    Err     = "Red"
    Label   = "Magenta"
    Muted   = "Gray"
}

# â”€â”€ Logging Functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Write-Step {
    param([string]$Text)
    Write-Host "`n  >> $Text" -ForegroundColor $global:ScriptColors.Info
}

function Write-Ok {
    param([string]$Text)
    Write-Host "     âœ“ $Text" -ForegroundColor $global:ScriptColors.Ok
}

function Write-Warn {
    param([string]$Text)
    Write-Host "     âš  $Text" -ForegroundColor $global:ScriptColors.Warn
}

function Write-Error {
    param([string]$Text)
    Write-Host "     âœ— $Text" -ForegroundColor $global:ScriptColors.Err
}

function Write-InfoMsg {
    param([string]$Text)
    Write-Host "     â„¹ $Text" -ForegroundColor $global:ScriptColors.Info
}

function Write-Muted {
    param([string]$Text)
    Write-Host "     $Text" -ForegroundColor $global:ScriptColors.Muted
}

# â”€â”€ Progress Indicator â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
$global:ProgressChars = @('/', '-', '\', '|')
$global:ProgressIndex = 0

function Write-Spinner {
    param([string]$Message)
    $char = $global:ProgressChars[$global:ProgressIndex % $global:ProgressChars.Length]
    Write-Host "`r  $char $Message" -NoNewline
    $global:ProgressIndex++
}

function Write-Progress {
    param(
        [string]$Activity,
        [int]$PercentComplete
    )
    $barLength = 30
    $filled = [math]::Floor($barLength * $PercentComplete / 100)
    $empty = $barLength - $filled
    $bar = "=" * $filled + " " * $empty
    Write-Host "`r  [$bar] $PercentComplete% - $Activity" -NoNewline
}

function Clear-Progress {
    Write-Host "`r$(' ' * 80)`r" -NoNewline
}

# â”€â”€ Error Handling â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Invoke-WithErrorHandling {
    param(
        [scriptblock]$ScriptBlock,
        [string]$ErrorMessage = "Operation failed",
        [switch]$ContinueOnError
    )

    try {
        & $ScriptBlock
        return $true
    } catch {
        Write-Error "${ErrorMessage}: $($_.Exception.Message)"
        if (-not $ContinueOnError) {
            throw
        }
        return $false
    }
}

# â”€â”€ Retry Logic â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Invoke-WithRetry {
    param(
        [scriptblock]$ScriptBlock,
        [int]$MaxRetries = 3,
        [int]$DelaySeconds = 2,
        [string]$OperationName = "Operation"
    )

    $attempt = 0
    while ($attempt -lt $MaxRetries) {
        try {
            & $ScriptBlock
            return
        } catch {
            $attempt++
            if ($attempt -lt $MaxRetries) {
                Write-Warn "$OperationName failed (attempt $attempt/$MaxRetries), retrying in ${DelaySeconds}s..."
                Start-Sleep -Seconds $DelaySeconds
                $DelaySeconds *= 2
            } else {
                Write-Error "$OperationName failed after $MaxRetries attempts: $($_.Exception.Message)"
                throw
            }
        }
    }
}

# â”€â”€ Process Management â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
$global:ManagedProcesses = @{}

function Start-TrackedProcess {
    param(
        [string]$Name,
        [string]$Path,
        [string[]]$Arguments = @(),
        [switch]$Hidden
    )

    $processParams = @{
        FilePath = $Path
        PassThru = $true
    }

    if ($Hidden) {
        $processParams.WindowStyle = "Hidden"
    }

    if ($Arguments.Count -gt 0) {
        $processParams.ArgumentList = $Arguments
    }

    $proc = Start-Process @processParams
    $global:ManagedProcesses[$Name] = $proc.Id
    return $proc
}

function Stop-TrackedProcess {
    param([string]$Name)

    if ($global:ManagedProcesses.ContainsKey($Name)) {
        $pid = $global:ManagedProcesses[$Name]
        try {
            $p = Get-Process -Id $pid -ErrorAction SilentlyContinue
            if ($p) {
                $p.Kill()
                Write-Ok "Stopped $Name (PID $pid)"
            }
        } catch {
            # Process already gone
        }
        $global:ManagedProcesses.Remove($Name)
    }
}

function Stop-AllTrackedProcesses {
    foreach ($name in $global:ManagedProcesses.Keys) {
        Stop-TrackedProcess -Name $name
    }
}

# â”€â”€ Cleanup Handler â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
$global:CleanupHandlers = @()

function Register-Cleanup {
    param([scriptblock]$Handler)
    $global:CleanupHandlers += $Handler
}

function Invoke-Cleanup {
    foreach ($handler in $global:CleanupHandlers) {
        try {
            & $handler
        } catch {
            Write-Warn "Cleanup handler failed: $($_.Exception.Message)"
        }
    }
}

# Register global cleanup handler
$null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action {
    Invoke-Cleanup
}

# â”€â”€ Configuration Management â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Get-ScriptConfig {
    param([string]$ConfigPath = "$PSScriptRoot/config.json")

    if (Test-Path $ConfigPath) {
        return Get-Content $ConfigPath -Raw | ConvertFrom-Json
    } else {
        return @{}
    }
}

function Set-ScriptConfig {
    param(
        [hashtable]$Config,
        [string]$ConfigPath = "$PSScriptRoot/config.json"
    )

    $Config | ConvertTo-Json -Depth 10 | Set-Content $ConfigPath
}

# â”€â”€ Validation Functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Test-Command {
    param([string]$Name)
    $null = Get-Command $Name -ErrorAction SilentlyContinue
    return $?
}

function Test-FileExists {
    param([string]$Path)
    if (Test-Path $Path) {
        return $true
    } else {
        Write-Warn "File not found: $Path"
        return $false
    }
}

function Test-PortAvailable {
    param(
        [int]$Port,
        [string]$Host = "localhost"
    )

    try {
        $tcp = New-Object System.Net.Sockets.TcpClient
        $tcp.Connect($Host, $Port)
        $tcp.Close()
        return $false
    } catch {
        return $true
    }
}

# â”€â”€ Timing Functions â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Start-Timer {
    return [System.Diagnostics.Stopwatch]::StartNew()
}

function Stop-Timer {
    param([System.Diagnostics.Stopwatch]$Timer)
    $Timer.Stop()
    return $Timer.Elapsed
}

function Format-Duration {
    param([TimeSpan]$Duration)

    if ($Duration.TotalMinutes -ge 1) {
        return "$([int]$Duration.TotalMinutes)m $([int]$Duration.TotalSeconds % 60)s"
    } else {
        return "$([int]$Duration.TotalSeconds)s"
    }
}

# â”€â”€ HTTP Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Invoke-Download {
    param(
        [string]$Url,
        [string]$OutPath,
        [string]$Description = "file"
    )

    Write-InfoMsg "Downloading $Description..."
    Invoke-WithRetry -OperationName "Download" -ScriptBlock {
        Invoke-WebRequest -Uri $Url -OutFile $OutPath -UseBasicParsing
    }
}

# â”€â”€ File Hash Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Get-FileHashEx {
    param(
        [string]$Path,
        [string]$Algorithm = "SHA256"
    )

    if (-not (Test-Path $Path)) {
        throw "File not found: $Path"
    }

    $hash = Get-FileHash -Path $Path -Algorithm $Algorithm -ErrorAction Stop
    return $hash.Hash
}

# â”€â”€ Directory Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Ensure-DirectoryExists {
    param(
        [string]$Path,
        [string]$Description = "directory"
    )

    if (-not (Test-Path $Path)) {
        try {
            New-Item -ItemType Directory -Path $Path -Force -ErrorAction Stop | Out-Null
            Write-Ok "Created ${Description}: $Path"
        } catch {
            Write-Error "Failed to create ${Description}: $($_.Exception.Message)"
            throw
        }
    }
}

# â”€â”€ Process Validation â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Test-ProcessRunning {
    param(
        [string]$ProcessName
    )

    $process = Get-Process -Name $ProcessName -ErrorAction SilentlyContinue
    return $null -ne $process
}

# â”€â”€ Network Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Test-UrlAccessible {
    param(
        [string]$Url,
        [int]$TimeoutSeconds = 30
    )

    try {
        $response = Invoke-WebRequest -Uri $Url -Method Head -UseBasicParsing -TimeoutSec $TimeoutSeconds -ErrorAction Stop
        return $response.StatusCode -eq 200
    } catch {
        return $false
    }
}

# â”€â”€ Version Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Test-VersionFormat {
    param(
        [string]$Version,
        [string]$Pattern = "^\d+\.\d+\.\d+$"
    )

    return $Version -match $Pattern
}

# â”€â”€ Build Utilities â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Get-PackageSourceHash {
    param(
        [string]$PackagePath
    )

    if (-not (Test-Path $PackagePath)) {
        return $null
    }

    $sourceFiles = Get-ChildItem -Path $PackagePath -Recurse -Filter "*.rs" -ErrorAction SilentlyContinue
    if ($sourceFiles) {
        return ($sourceFiles | ForEach-Object { $_.LastWriteTime.Ticks } | Measure-Object -Sum).Sum
    }

    return $null
}

# â”€â”€ Logging to File â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Start-FileLogging {
    param(
        [string]$LogPath,
        [switch]$Append
    )

    if ($Append) {
        Start-Transcript -Path $LogPath -Append
    } else {
        Start-Transcript -Path $LogPath
    }

    Register-Cleanup -Handler {
        Stop-Transcript -ErrorAction SilentlyContinue
    }
}

# â”€â”€ Module Information â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€
function Get-ScriptInfo {
    $scriptPath = $MyInvocation.PSCommandPath
    $scriptDir = Split-Path $scriptPath -Parent
    $rootDir = Split-Path $scriptDir -Parent

    return @{
        ScriptPath = $scriptPath
        ScriptDir  = $scriptDir
        RootDir    = $rootDir
    }
}

