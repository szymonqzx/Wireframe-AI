# Environment Validation Script for Wireframe-AI
# Ensures agent environment matches team setup

Write-Host "🔍 Validating Wireframe-AI Environment..." -ForegroundColor Cyan

$errors = 0
$warnings = 0

# Check Rust installation
Write-Host "`n📦 Checking Rust installation..." -ForegroundColor Yellow
$rustVersion = rustc --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Rust installed: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Rust not found" -ForegroundColor Red
    $errors++
}

# Check Cargo
Write-Host "`n📦 Checking Cargo..." -ForegroundColor Yellow
$cargoVersion = cargo --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Cargo installed: $cargoVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Cargo not found" -ForegroundColor Red
    $errors++
}

# Check NATS server
Write-Host "`n📦 Checking NATS server..." -ForegroundColor Yellow
$natsVersion = nats-server -v 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ NATS server installed" -ForegroundColor Green
} else {
    Write-Host "⚠ NATS server not found (may need to install)" -ForegroundColor Yellow
    $warnings++
}

# Check Python (for adapters)
Write-Host "`n📦 Checking Python..." -ForegroundColor Yellow
$pythonVersion = python --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Python installed: $pythonVersion" -ForegroundColor Green
} else {
    Write-Host "⚠ Python not found (needed for adapters)" -ForegroundColor Yellow
    $warnings++
}

# Check workspace dependencies
Write-Host "`n📦 Checking workspace dependencies..." -ForegroundColor Yellow
if (Test-Path "Cargo.toml") {
    Write-Host "✓ Cargo.toml found" -ForegroundColor Green
    
    # Try to check if dependencies are available
    cargo check --quiet 2>$null | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✓ Dependencies are available" -ForegroundColor Green
    } else {
        Write-Host "⚠ Dependencies may need to be fetched (run cargo fetch)" -ForegroundColor Yellow
        $warnings++
    }
} else {
    Write-Host "✗ Cargo.toml not found" -ForegroundColor Red
    $errors++
}

# Check for pre-commit hooks
Write-Host "`n📦 Checking pre-commit hooks..." -ForegroundColor Yellow
if (Test-Path ".git/hooks/pre-commit") {
    Write-Host "✓ Pre-commit hooks found" -ForegroundColor Green
} else {
    Write-Host "⚠ Pre-commit hooks not configured" -ForegroundColor Yellow
    $warnings++
}

# Check for essential tools
Write-Host "`n📦 Checking essential tools..." -ForegroundColor Yellow

# Check git
$gitVersion = git --version 2>$null
if ($LASTEXITCODE -eq 0) {
    Write-Host "✓ Git installed: $gitVersion" -ForegroundColor Green
} else {
    Write-Host "✗ Git not found" -ForegroundColor Red
    $errors++
}

# Check for development scripts
Write-Host "`n📦 Checking development scripts..." -ForegroundColor Yellow
if (Test-Path ".devin/scripts") {
    Write-Host "✓ Devin scripts directory found" -ForegroundColor Green
} else {
    Write-Host "⚠ Devin scripts directory not found" -ForegroundColor Yellow
    $warnings++
}

# Check for documentation
Write-Host "`n📦 Checking documentation..." -ForegroundColor Yellow
if (Test-Path "AGENTS.md") {
    Write-Host "✓ AGENTS.md found" -ForegroundColor Green
} else {
    Write-Host "⚠ AGENTS.md not found" -ForegroundColor Yellow
    $warnings++
}

if (Test-Path "docs") {
    Write-Host "✓ Documentation directory found" -ForegroundColor Green
} else {
    Write-Host "⚠ Documentation directory not found" -ForegroundColor Yellow
    $warnings++
}

# Summary
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "Environment Validation Summary" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

if ($errors -eq 0 -and $warnings -eq 0) {
    Write-Host "✓ All checks passed! Environment is ready." -ForegroundColor Green
    exit 0
} elseif ($errors -eq 0) {
    Write-Host "⚠ Validation passed with $warnings warning(s)" -ForegroundColor Yellow
    Write-Host "  You may want to address the warnings for optimal development experience." -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "✗ Validation failed with $errors error(s) and $warnings warning(s)" -ForegroundColor Red
    Write-Host "  Please fix the errors before proceeding." -ForegroundColor Red
    exit 1
}
