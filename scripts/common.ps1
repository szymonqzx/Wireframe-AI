#!/usr/bin/env pwsh
# TEAM_006: Common utilities and functions for Wireframe-AI scripts
# Provides centralized error handling, logging, progress indicators, and retry logic

$ErrorActionPreference = "Stop"

# -- Color Configuration --
$global:ScriptColors = @{
    Info    = "Cyan"
    Ok      = "Green"
    Warn    = "Yellow"
    Err     = "Red"
    Label   = "Magenta"
    Muted   = "Gray"
}

# -- Logging Functions --
function Write-Step {
    param([string]$Text)
    Write-Host "`n  >> $Text" -ForegroundColor $global:ScriptColors.Info
}

function Write-Ok {
    param([string]$Text)
    Write-Host "     [OK] $Text" -ForegroundColor $global:ScriptColors.Ok
}

function Write-Warn {
    param([string]$Text)
    Write-Host "     [WARN] $Text" -ForegroundColor $global:ScriptColors.Warn
}

function Write-Error {
    param([string]$Text)
    Write-Host "     [ERROR] $Text" -ForegroundColor $global:ScriptColors.Err
}

function Write-InfoMsg {
    param([string]$Text)
    Write-Host "     [INFO] $Text" -ForegroundColor $global:ScriptColors.Info
}

function Write-Muted {
    param([string]$Text)
    Write-Host "     $Text" -ForegroundColor $global:ScriptColors.Muted
}

# -- Progress Indicator --
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

# -- Error Handling --
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

# -- Retry Logic --
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

# -- Process Management --
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

# -- Cleanup Handler --
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

# -- Configuration Management --
function Get-ConfigValue {
    param(
        [string]$Key,
        [string]$Default = ""
    )
    $envValue = [Environment]::GetEnvironmentVariable($Key)
    if (-not [string]::IsNullOrWhiteSpace($envValue)) {
        return $envValue
    }
    return $Default
}

function Set-ConfigValue {
    param(
        [string]$Key,
        [string]$Value
    )
    [Environment]::SetEnvironmentVariable($Key, $Value, "User")
}

# -- Command Validation --
function Test-Command {
    param([string]$Command)
    try {
        $null = Get-Command $Command -ErrorAction Stop
        return $true
    } catch {
        return $false
    }
}

# -- Version Checking --
function Test-MinimumVersion {
    param(
        [string]$CurrentVersion,
        [string]$MinimumVersion
    )

    $current = [version]$CurrentVersion
    $minimum = [version]$MinimumVersion

    return $current -ge $minimum
}

function Get-VersionNumber {
    param(
        [string]$VersionString,
        [string]$Pattern = "\d+\.\d+\.\d+"
    )

    if ($VersionString -match $Pattern) {
        return $matches[0]
    }

    return $null
}

# -- Build Utilities --
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

# -- Logging to File --
function Start-FileLogging {
    param(
        [string]$LogPath,
        [switch]$Append
    )

    $logDir = Split-Path $LogPath -Parent
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir -Force | Out-Null
    }

    if ($Append) {
        Start-Transcript -Path $LogPath -Append
    } else {
        Start-Transcript -Path $LogPath
    }
}

function Stop-FileLogging {
    Stop-Transcript
}

# -- Network Utilities --
function Test-Port {
    param(
        [string]$HostName,
        [int]$Port,
        [int]$TimeoutMs = 5000
    )

    try {
        $tcpClient = New-Object System.Net.Sockets.TcpClient
        $connectTask = $tcpClient.ConnectAsync($HostName, $Port)
        $completed = $connectTask.Wait($TimeoutMs)

        if ($completed) {
            $tcpClient.Close()
            return $true
        } else {
            $tcpClient.Close()
            return $false
        }
    } catch {
        return $false
    }
}

# -- Directory Utilities --
function Ensure-Directory {
    param([string]$Path)

    if (-not (Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Remove-DirectorySafely {
    param(
        [string]$Path,
        [switch]$Recurse
    )

    if (Test-Path $Path) {
        Remove-Item -Path $Path -Force:$Recurse -Recurse:$Recurse
    }
}

# -- File Utilities --
function Get-FileHashSafe {
    param(
        [string]$Path,
        [string]$Algorithm = "SHA256"
    )

    if (Test-Path $Path) {
        try {
            return (Get-FileHash -Path $Path -Algorithm $Algorithm).Hash
        } catch {
            return $null
        }
    }
    return $null
}

# -- String Utilities --
function New-GuidString {
    return [Guid]::NewGuid().ToString()
}

function Get-Timestamp {
    return Get-Date -Format "yyyy-MM-dd HH:mm:ss"
}

# -- Platform Detection --
function Test-Windows {
    return $PSVersionTable.Platform -eq "Win32NT" -or $IsWindows
}

function Test-Linux {
    return $PSVersionTable.Platform -eq "Unix" -and $IsLinux
}

function Test-MacOS {
    return $PSVersionTable.Platform -eq "Unix" -and $IsMacOS
}

# -- Timer Functions --
function Start-Timer {
    return Get-Date
}

function Stop-Timer {
    param(
        [DateTime]$Timer
    )
    $endTime = Get-Date
    return $endTime - $Timer
}

function Format-Duration {
    param(
        [TimeSpan]$Duration
    )

    if ($Duration.TotalSeconds -lt 1) {
        return "$($Duration.TotalMilliseconds.ToString('0'))ms"
    } elseif ($Duration.TotalMinutes -lt 1) {
        return "$($Duration.TotalSeconds.ToString('0.0'))s"
    } elseif ($Duration.TotalHours -lt 1) {
        return "$($Duration.TotalMinutes.ToString('0.0'))m"
    } else {
        return "$($Duration.TotalHours.ToString('0.0'))h"
    }
}

function Test-ProcessRunning {
    param(
        [string]$ProcessName
    )
    
    $processes = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -like "$ProcessName*" }
    return $processes -ne $null
}
