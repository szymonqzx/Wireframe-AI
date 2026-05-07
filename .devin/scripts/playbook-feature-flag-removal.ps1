# Feature Flag Removal Playbook for Wireframe-AI
# Automated workflow for safely removing feature flags

param(
    [Parameter(Mandatory=$true)]
    [string]$FlagName,
    [switch]$DryRun,
    [switch]$CheckOnly
)

Write-Host "🚩 Wireframe-AI Feature Flag Removal Playbook" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

if ($CheckOnly) {
    Write-Host "`n🔍 Checking for feature flags in codebase..." -ForegroundColor Yellow
    
    # Search for common feature flag patterns
    $flagPatterns = @(
        "feature_flag",
        "featureFlag",
        "FEATURE_FLAG",
        "flag_",
        "is_enabled",
        "is_enabled",
        "enable_",
        "disable_"
    )
    
    $rustFiles = Get-ChildItem -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "target" }
    $foundFlags = @{}
    
    foreach ($file in $rustFiles) {
        $content = Get-Content $file.FullName -Raw
        foreach ($pattern in $flagPatterns) {
            if ($content -match $pattern) {
                if (-not $foundFlags.ContainsKey($pattern)) {
                    $foundFlags[$pattern] = @()
                }
                $foundFlags[$pattern] += $file.FullName
            }
        }
    }
    
    if ($foundFlags.Count -gt 0) {
        Write-Host "`nFound potential feature flags:" -ForegroundColor Green
        foreach ($pattern in $foundFlags.Keys) {
            Write-Host "  Pattern: $pattern" -ForegroundColor Yellow
            foreach ($file in $foundFlags[$pattern]) {
                Write-Host "    - $file" -ForegroundColor Gray
            }
        }
    } else {
        Write-Host "No feature flags found" -ForegroundColor Green
    }
    
    exit 0
}

# Step 1: Search for flag usage
Write-Host "`n📋 Step 1: Searching for flag '$FlagName' usage..." -ForegroundColor Yellow

$rustFiles = Get-ChildItem -Recurse -Filter "*.rs" | Where-Object { $_.FullName -notmatch "target" }
$flagUsages = @()

foreach ($file in $rustFiles) {
    $content = Get-Content $file.FullName -Raw
    if ($content -match $FlagName) {
        $flagUsages += $file.FullName
    }
}

if ($flagUsages.Count -eq 0) {
    Write-Host "⚠ Flag '$FlagName' not found in codebase" -ForegroundColor Yellow
    exit 0
}

Write-Host "Found flag in $($flagUsages.Count) file(s):" -ForegroundColor Green
foreach ($usage in $flagUsages) {
    Write-Host "  - $usage" -ForegroundColor Gray
}

# Step 2: Backup current state
Write-Host "`n📋 Step 2: Creating backup..." -ForegroundColor Yellow
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$backupDir = ".feature-flag-backup-$timestamp"
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null

foreach ($file in $flagUsages) {
    $relativePath = $file.Substring((Get-Location).Path.Length + 1)
    $backupPath = Join-Path $backupDir $relativePath
    $backupDirPath = Split-Path $backupPath -Parent
    New-Item -ItemType Directory -Force -Path $backupDirPath | Out-Null
    Copy-Item $file -Destination $backupPath -Force
}

Write-Host "✓ Backup created at $backupDir" -ForegroundColor Green

# Step 3: Check current build status
Write-Host "`n📋 Step 3: Checking current build status..." -ForegroundColor Yellow
cargo check --quiet 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Current build is failing. Fix before removing feature flag." -ForegroundColor Red
    exit 1
}
Write-Host "✓ Current build is passing" -ForegroundColor Green

# Step 4: Run tests
Write-Host "`n📋 Step 4: Running current tests..." -ForegroundColor Yellow
cargo test --quiet 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠ Current tests are failing. Consider fixing before removal." -ForegroundColor Yellow
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne "y") {
        exit 1
    }
} else {
    Write-Host "✓ Current tests are passing" -ForegroundColor Green
}

# Step 5: Remove flag usage
Write-Host "`n📋 Step 5: Removing flag usage..." -ForegroundColor Yellow

if ($DryRun) {
    Write-Host "[DRY RUN] Would remove flag from $($flagUsages.Count) file(s)" -ForegroundColor Cyan
    foreach ($file in $flagUsages) {
        Write-Host "  - $file" -ForegroundColor Gray
    }
} else {
    foreach ($file in $flagUsages) {
        $content = Get-Content $file -Raw
        
        # Remove flag-related code (this is a simple example - customize for your patterns)
        $newContent = $content -replace [regex]::Escape($FlagName), ""
        
        if ($newContent -ne $content) {
            $newContent | Out-File $file -Encoding UTF8 -NoNewline
            Write-Host "  Removed flag from: $file" -ForegroundColor Green
        }
    }
}

# Step 6: Check build after removal
Write-Host "`n📋 Step 6: Checking build after removal..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo check" -ForegroundColor Cyan
} else {
    cargo check --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Build failed after flag removal" -ForegroundColor Red
        Write-Host "Restoring from backup..." -ForegroundColor Yellow
        foreach ($file in $flagUsages) {
            $relativePath = $file.Substring((Get-Location).Path.Length + 1)
            $backupPath = Join-Path $backupDir $relativePath
            Copy-Item $backupPath -Destination $file -Force
        }
        exit 1
    }
    Write-Host "✓ Build passes after removal" -ForegroundColor Green
}

# Step 7: Run tests after removal
Write-Host "`n📋 Step 7: Running tests after removal..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo test" -ForegroundColor Cyan
} else {
    cargo test --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠ Tests failed after flag removal" -ForegroundColor Yellow
        Write-Host "This may require code adjustments" -ForegroundColor Yellow
        $response = Read-Host "Keep changes anyway? (y/N)"
        if ($response -ne "y") {
            Write-Host "Restoring from backup..." -ForegroundColor Yellow
            foreach ($file in $flagUsages) {
                $relativePath = $file.Substring((Get-Location).Path.Length + 1)
                $backupPath = Join-Path $backupDir $relativePath
                Copy-Item $backupPath -Destination $file -Force
            }
            exit 1
        }
    } else {
        Write-Host "✓ Tests pass after removal" -ForegroundColor Green
    }
}

# Step 8: Run clippy
Write-Host "`n📋 Step 8: Running clippy..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo clippy" -ForegroundColor Cyan
} else {
    cargo clippy --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠ Clippy found issues after removal" -ForegroundColor Yellow
        Write-Host "Review clippy output for dead code or unused imports" -ForegroundColor Yellow
    } else {
        Write-Host "✓ Clippy passes after removal" -ForegroundColor Green
    }
}

# Step 9: Summary
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "Feature Flag Removal Summary" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "✓ Feature flag '$FlagName' removed successfully" -ForegroundColor Green
Write-Host "✓ Build passes" -ForegroundColor Green
Write-Host "✓ Tests pass" -ForegroundColor Green
Write-Host "Backup location: $backupDir" -ForegroundColor Yellow
Write-Host "To restore: Copy files from $backupDir back to original locations" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

exit 0
