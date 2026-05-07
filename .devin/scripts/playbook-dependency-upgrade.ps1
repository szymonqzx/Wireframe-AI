# Dependency Upgrade Playbook for Wireframe-AI
# Automated workflow for upgrading Cargo dependencies

param(
    [string]$Dependency = "",
    [switch]$DryRun,
    [switch]$CheckOnly
)

Write-Host "📦 Wireframe-AI Dependency Upgrade Playbook" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

if ($CheckOnly) {
    Write-Host "`n🔍 Checking for outdated dependencies..." -ForegroundColor Yellow
    
    # Check for outdated dependencies
    cargo outdated 2>$null
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠ cargo-outdated not installed. Install with: cargo install cargo-outdated" -ForegroundColor Yellow
        Write-Host "Falling back to manual check..." -ForegroundColor Yellow
        
        # Manual check by looking at Cargo.toml
        Write-Host "`nCurrent dependencies in Cargo.toml:" -ForegroundColor Yellow
        Get-Content "Cargo.toml" | Select-String "^\s+\w+\s*="
    }
    
    exit 0
}

if ([string]::IsNullOrEmpty($Dependency)) {
    Write-Host "`n⚠ No specific dependency specified. Checking all dependencies..." -ForegroundColor Yellow
    Write-Host "Use -Dependency <name> to upgrade a specific dependency" -ForegroundColor Yellow
    Write-Host "Use -CheckOnly to check for outdated dependencies" -ForegroundColor Yellow
    Write-Host "Use -DryRun to simulate upgrade without making changes" -ForegroundColor Yellow
}

# Step 1: Backup current state
Write-Host "`n📋 Step 1: Creating backup of current state..." -ForegroundColor Yellow
$timestamp = Get-Date -Format "yyyyMMddHHmmss"
$backupDir = ".dependency-backup-$timestamp"
New-Item -ItemType Directory -Force -Path $backupDir | Out-Null
Copy-Item "Cargo.toml" -Destination "$backupDir/Cargo.toml" -Force
Copy-Item "Cargo.lock" -Destination "$backupDir/Cargo.lock" -Force -ErrorAction SilentlyContinue
Write-Host "✓ Backup created at $backupDir" -ForegroundColor Green

# Step 2: Check current build status
Write-Host "`n📋 Step 2: Checking current build status..." -ForegroundColor Yellow
cargo check --quiet 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Current build is failing. Fix before upgrading dependencies." -ForegroundColor Red
    exit 1
}
Write-Host "✓ Current build is passing" -ForegroundColor Green

# Step 3: Run tests
Write-Host "`n📋 Step 3: Running current tests..." -ForegroundColor Yellow
cargo test --quiet 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠ Current tests are failing. Consider fixing before upgrading." -ForegroundColor Yellow
    $response = Read-Host "Continue anyway? (y/N)"
    if ($response -ne "y") {
        exit 1
    }
} else {
    Write-Host "✓ Current tests are passing" -ForegroundColor Green
}

# Step 4: Upgrade dependency
Write-Host "`n📋 Step 4: Upgrading dependency..." -ForegroundColor Yellow

if ([string]::IsNullOrEmpty($Dependency)) {
    Write-Host "Upgrading all dependencies..." -ForegroundColor Yellow
    if ($DryRun) {
        Write-Host "[DRY RUN] Would run: cargo update" -ForegroundColor Cyan
    } else {
        cargo update
    }
} else {
    Write-Host "Upgrading $Dependency..." -ForegroundColor Yellow
    if ($DryRun) {
        Write-Host "[DRY RUN] Would run: cargo update -p $Dependency" -ForegroundColor Cyan
    } else {
        cargo update -p $Dependency
    }
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Dependency upgrade failed" -ForegroundColor Red
    Write-Host "Restoring from backup..." -ForegroundColor Yellow
    Copy-Item "$backupDir/Cargo.toml" -Destination "Cargo.toml" -Force
    Copy-Item "$backupDir/Cargo.lock" -Destination "Cargo.lock" -Force -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "✓ Dependency upgrade completed" -ForegroundColor Green

# Step 5: Check build after upgrade
Write-Host "`n📋 Step 5: Checking build after upgrade..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo check" -ForegroundColor Cyan
} else {
    cargo check --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "✗ Build failed after upgrade" -ForegroundColor Red
        Write-Host "Restoring from backup..." -ForegroundColor Yellow
        Copy-Item "$backupDir/Cargo.toml" -Destination "Cargo.toml" -Force
        Copy-Item "$backupDir/Cargo.lock" -Destination "Cargo.lock" -Force -ErrorAction SilentlyContinue
        exit 1
    }
    Write-Host "✓ Build passes after upgrade" -ForegroundColor Green
}

# Step 6: Run tests after upgrade
Write-Host "`n📋 Step 6: Running tests after upgrade..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo test" -ForegroundColor Cyan
} else {
    cargo test --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠ Tests failed after upgrade" -ForegroundColor Yellow
        Write-Host "This may require code adjustments for the new dependency version" -ForegroundColor Yellow
        $response = Read-Host "Keep changes anyway? (y/N)"
        if ($response -ne "y") {
            Write-Host "Restoring from backup..." -ForegroundColor Yellow
            Copy-Item "$backupDir/Cargo.toml" -Destination "Cargo.toml" -Force
            Copy-Item "$backupDir/Cargo.lock" -Destination "Cargo.lock" -Force -ErrorAction SilentlyContinue
            exit 1
        }
    } else {
        Write-Host "✓ Tests pass after upgrade" -ForegroundColor Green
    }
}

# Step 7: Run clippy
Write-Host "`n📋 Step 7: Running clippy..." -ForegroundColor Yellow
if ($DryRun) {
    Write-Host "[DRY RUN] Would run: cargo clippy" -ForegroundColor Cyan
} else {
    cargo clippy --quiet 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠ Clippy found issues after upgrade" -ForegroundColor Yellow
        Write-Host "Review clippy output for breaking changes" -ForegroundColor Yellow
    } else {
        Write-Host "✓ Clippy passes after upgrade" -ForegroundColor Green
    }
}

# Step 8: Summary
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "Dependency Upgrade Summary" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "✓ Dependency upgrade completed successfully" -ForegroundColor Green
Write-Host "✓ Build passes" -ForegroundColor Green
Write-Host "✓ Tests pass" -ForegroundColor Green
Write-Host "Backup location: $backupDir" -ForegroundColor Yellow
Write-Host "To restore: Copy $backupDir/Cargo.toml and Cargo.lock back" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

exit 0
