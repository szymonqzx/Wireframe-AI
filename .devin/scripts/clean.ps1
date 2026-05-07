#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Clean build artifacts and temporary files from Wireframe-AI project.

.DESCRIPTION
    This script removes build artifacts, temporary files, and coverage reports
    to keep the project directory clean.

.PARAMETER All
    Clean everything including target directory, coverage, and temp files. Default: false

.PARAMETER Target
    Clean target directory. Default: true

.PARAMETER Coverage
    Clean coverage reports. Default: true

.PARAMETER Temp
    Clean temporary files. Default: true

.PARAMETER DryRun
    Show what would be deleted without actually deleting. Default: false
#>

param(
    [Parameter()]
    [switch]$All,

    [Parameter()]
    [switch]$Target,

    [Parameter()]
    [switch]$Coverage,

    [Parameter()]
    [switch]$Temp,

    [Parameter()]
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Remove-Safely {
    param(
        [string]$Path,
        [string]$Description
    )

    if (Test-Path $Path) {
        if ($DryRun) {
            Write-Host "[DRY RUN] Would remove: $Description ($Path)"
        } else {
            Write-Host "Removing: $Description ($Path)"
            Remove-Item -Recurse -Force $Path
        }
    } else {
        Write-Host "Not found (skipping): $Description ($Path)"
    }
}

function Invoke-CleanTarget {
    Write-Host "Cleaning target directory..."
    Remove-Safely -Path "target" -Description "Target directory"
}

function Invoke-CleanCoverage {
    Write-Host "Cleaning coverage reports..."
    Remove-Safely -Path "coverage" -Description "Coverage reports"
}

function Invoke-CleanTemp {
    Write-Host "Cleaning temporary files..."

    $tempPatterns = @(
        "*.profraw",
        "*.profdata",
        "*.pdb",
        "*.log"
    )

    foreach ($pattern in $tempPatterns) {
        $files = Get-ChildItem -Path . -Filter $pattern -Recurse -ErrorAction SilentlyContinue
        foreach ($file in $files) {
            if ($DryRun) {
                Write-Host "[DRY RUN] Would remove: $($file.FullName)"
            } else {
                Write-Host "Removing: $($file.FullName)"
                Remove-Item -Force $file.FullName
            }
        }
    }
}

function Invoke-CleanAll {
    Write-Host "Cleaning everything..."
    Invoke-CleanTarget
    Invoke-CleanCoverage
    Invoke-CleanTemp
}

# Main execution
Write-Host "=== Wireframe-AI Clean ==="
Write-Host ""

if ($All) {
    Invoke-CleanAll
} else {
    if ($Target) {
        Invoke-CleanTarget
    }
    if ($Coverage) {
        Invoke-CleanCoverage
    }
    if ($Temp) {
        Invoke-CleanTemp
    }
}

if ($DryRun) {
    Write-Host ""
    Write-Host "[DRY RUN] No files were actually deleted."
    Write-Host "Run without -DryRun to perform the cleanup."
} else {
    Write-Host ""
    Write-Host "Cleanup complete!"
}
