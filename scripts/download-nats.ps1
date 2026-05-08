#!/usr/bin/env pwsh
# TEAM_006: Download NATS server binary for the current platform with retry logic
# Stores in kernel/nats/
param(
    [string]$Version = "2.10.27",
    [switch]$Force
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
$natsDir = Join-Path (Join-Path (Join-Path $PSScriptRoot "..") "kernel") "nats"

Write-Host ""
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host "  |     Wireframe AI - Download NATS Server        |" -ForegroundColor $global:ScriptColors.Label
Write-Host "  +--------------------------------------------------+" -ForegroundColor $global:ScriptColors.Label
Write-Host ""

Write-Step "Downloading NATS server..."
Write-Muted "Version: $Version"

# Validate version format
if ($Version -notmatch "^\d+\.\d+\.\d+$") {
    Write-Error "Invalid version format: $Version. Expected format: X.Y.Z"
    exit 1
}

# Create directory with error handling
try {
    if (!(Test-Path $natsDir)) {
        New-Item -ItemType Directory -Path $natsDir -Force -ErrorAction Stop | Out-Null
        Write-Ok "Created directory: $natsDir"
    }
} catch {
    Write-Error "Failed to create directory ${natsDir}: ${_}"
    exit 1
}

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "amd64" } else { "386" }
$os = if ($IsLinux) { "linux" } elseif ($IsMacOS) { "darwin" } else { "windows" }
$ext = if ($os -eq "windows") { "zip" } else { "tar.gz" }

$exeName = if ($os -eq "windows") { "nats-server.exe" } else { "nats-server" }
$exePath = Join-Path $natsDir $exeName

# Check if already exists
if ((Test-Path $exePath) -and -not $Force) {
    Write-InfoMsg "NATS server already exists at $exePath"
    Write-InfoMsg "Use -Force to re-download"
    exit 0
}

$url = "https://github.com/nats-io/nats-server/releases/download/v${Version}/nats-server-v${Version}-${os}-${arch}.${ext}"
$outFile = Join-Path $natsDir "nats-server.${ext}"

Write-InfoMsg "Platform: $os-$arch"
Write-InfoMsg "Downloading from GitHub..."

$downloadTimer = Start-Timer

try {
    # Validate URL format
    if (-not ($url -match "^https://")) {
        Write-Error "Invalid URL format: $url"
        exit 1
    }

    # Download file with progress tracking
    try {
        $webClient = New-Object System.Net.WebClient
        $webClient.DownloadFile($url, $outFile)
        Write-Ok "Downloaded NATS server v${Version}"
    } catch {
        Write-Error "Failed to download: ${_}"
        throw
    }

    $downloadDuration = Stop-Timer -Timer $downloadTimer
    $durationStr = Format-Duration -Duration $downloadDuration
    Write-Ok "Download completed in $durationStr"

    Write-Step "Extracting archive..."

    try {
        if ($ext -eq "zip") {
            Expand-Archive -Path $outFile -DestinationPath $natsDir -Force -ErrorAction Stop
            $exe = Get-ChildItem -Path $natsDir -Recurse -Filter $exeName -ErrorAction SilentlyContinue | Select-Object -First 1
        } else {
            tar -xzf $outFile -C $natsDir
            if ($LASTEXITCODE -ne 0) {
                throw "tar extraction failed with exit code $LASTEXITCODE"
            }
            $exe = Get-ChildItem -Path $natsDir -Recurse -Filter $exeName -ErrorAction SilentlyContinue | Select-Object -First 1
        }

        if ($exe) {
            Copy-Item $exe.FullName (Join-Path $natsDir $exeName) -Force -ErrorAction Stop
            Write-Ok "NATS server installed to $exePath"

            # Verify the executable
            if (Test-Path $exePath) {
                Write-Ok "Executable verified at $exePath"
            } else {
                throw "Executable not found after copy"
            }
        } else {
            throw "Failed to find $exeName in downloaded archive"
        }

        Remove-Item $outFile -Force -ErrorAction SilentlyContinue
        Write-Ok "Cleanup complete"

        Write-Host ""
        Write-Step "Installation summary"
        Write-Ok "NATS server v$Version installed successfully"
        Write-Muted "Location: $exePath"
        Write-Muted "Platform: $os-$arch"

        exit 0
    } catch {
        Write-Error "Extraction failed: ${_}"
        throw
    }
} catch {
    Write-Error "Failed to download/extract NATS server: ${_}"

    # Cleanup on failure
    if (Test-Path $outFile) {
        Remove-Item $outFile -Force -ErrorAction SilentlyContinue
    }

    exit 1
}
