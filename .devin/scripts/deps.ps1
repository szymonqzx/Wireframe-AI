#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Dependency management script for Wireframe-AI project.

.DESCRIPTION
    This script checks, installs, and updates project dependencies including
    Rust tools, NATS server, and other required components.

.PARAMETER Check
    Check if all dependencies are installed. Default: true

.PARAMETER Install
    Install missing dependencies. Default: false

.PARAMETER Update
    Update all dependencies. Default: false

.PARAMETER Rust
    Include Rust toolchain in operations. Default: true

.PARAMETER Nats
    Include NATS server in operations. Default: true

.PARAMETER Python
    Include Python dependencies in operations. Default: true
#>

param(
    [Parameter()]
    [switch]$Check,

    [Parameter()]
    [switch]$Install,

    [Parameter()]
    [switch]$Update,

    [Parameter()]
    [switch]$Rust,

    [Parameter()]
    [switch]$Nats,

    [Parameter()]
    [switch]$Python
)

$ErrorActionPreference = "Stop"

function Test-RustToolchain {
    Write-Host "Checking Rust toolchain..."

    $rustc = Get-Command rustc -ErrorAction SilentlyContinue
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue

    if ($rustc -and $cargo) {
        Write-Host "✓ Rust toolchain installed"
        Write-Host "  rustc: $(rustc --version 2>$(rustc --version)1)"
        Write-Host "  cargo: $(cargo --version 2>$(cargo --version)1)"
        return $true
    } else {
        Write-Host "✗ Rust toolchain not found"
        return $false
    }
}

function Test-NatsServer {
    Write-Host "Checking NATS server..."

    $natsPath = Join-Path $PSScriptRoot "..\..\kernel\nats\nats-server.exe"
    if (Test-Path $natsPath) {
        Write-Host "✓ NATS server installed at $natsPath"
        return $true
    } else {
        Write-Host "✗ NATS server not found"
        return $false
    }
}

function Test-PythonEnvironment {
    Write-Host "Checking Python environment..."

    $python = Get-Command python -ErrorAction SilentlyContinue
    if ($python) {
        Write-Host "✓ Python found at $($python.Source)"
        Write-Host "  Python: $(python --version 2>$(python --version)1)"
        return $true
    } else {
        Write-Host "✗ Python not found"
        return $false
    }
}

function Test-CargoTools {
    Write-Host "Checking cargo tools..."

    $tools = @{
        "cargo-tarpaulin" = "cargo-tarpaulin"
        "grcov" = "grcov"
    }

    $allInstalled = $true
    foreach ($tool in $tools.GetEnumerator()) {
        $installed = Get-Command $tool.Value -ErrorAction SilentlyContinue
        if ($installed) {
            Write-Host "✓ $($tool.Key) installed"
        } else {
            Write-Host "✗ $($tool.Key) not found"
            $allInstalled = $false
        }
    }

    return $allInstalled
}

function Install-RustToolchain {
    Write-Host "Installing Rust toolchain..."
    Write-Host "Please visit https://rustup.rs/ to install Rust"
    Write-Host "Or run: winget install Rustlang.Rustup"
}

function Install-NatsServer {
    Write-Host "Installing NATS server..."
    Write-Host "Running download-nats.ps1 script..."
    & "$PSScriptRoot\..\..\scripts\download-nats.ps1"
}

function Install-PythonDependencies {
    Write-Host "Installing Python dependencies..."

    $rootDir = Resolve-Path "$PSScriptRoot\..\.."
    $sdkDir = Join-Path $rootDir "sdk\agentic-sdk-py"
    $adapterDir = Join-Path $rootDir "adapter\python"

    Write-Host "Installing agentic-sdk-py..."
    Push-Location $rootDir
    try {
        pip install -e $sdkDir --quiet
        Write-Host "✓ agentic-sdk-py installed"
    } finally {
        Pop-Location
    }

    Write-Host "Installing wireframe-ai-adapter..."
    Push-Location $rootDir
    try {
        pip install -e $adapterDir --quiet
        Write-Host "✓ wireframe-ai-adapter installed"
    } finally {
        Pop-Location
    }
}

function Install-CargoTools {
    Write-Host "Installing cargo tools..."

    Write-Host "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin

    Write-Host "Installing grcov..."
    cargo install grcov
}

function Update-CargoTools {
    Write-Host "Updating cargo tools..."

    Write-Host "Updating cargo-tarpaulin..."
    cargo install cargo-tarpaulin --force

    Write-Host "Updating grcov..."
    cargo install grcov --force
}

function Update-RustToolchain {
    Write-Host "Updating Rust toolchain..."
    rustup update
}

# Main execution
Write-Host "=== Wireframe-AI Dependency Management ==="
Write-Host ""

$missingDeps = @()

if ($Rust) {
    if (-not (Test-RustToolchain)) {
        $missingDeps += "Rust toolchain"
    }
}

if ($Nats) {
    if (-not (Test-NatsServer)) {
        $missingDeps += "NATS server"
    }
}

if ($Python) {
    if (-not (Test-PythonEnvironment)) {
        $missingDeps += "Python environment"
    }
}

Test-CargoTools

if ($Install -and $missingDeps.Count -gt 0) {
    Write-Host ""
    Write-Host "Installing missing dependencies..."

    if ($missingDeps -contains "Rust toolchain") {
        Install-RustToolchain
    }

    if ($missingDeps -contains "NATS server") {
        Install-NatsServer
    }

    if ($missingDeps -contains "Python environment") {
        Install-PythonDependencies
    }

    Install-CargoTools
}

if ($Update) {
    Write-Host ""
    Write-Host "Updating dependencies..."

    if ($Rust) {
        Update-RustToolchain
    }

    Update-CargoTools
}

Write-Host ""
if ($missingDeps.Count -eq 0) {
    Write-Host "✓ All dependencies are installed!"
} else {
    Write-Host "Missing dependencies: $($missingDeps -join ', ')"
    Write-Host "Run with -Install to install missing dependencies."
}
