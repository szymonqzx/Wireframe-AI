# Automated Code Review Playbook for Wireframe-AI
# Checks for common mistakes, security issues, and code quality problems

param(
    [string]$Branch = "main",
    [switch]$Verbose
)

Write-Host "🔍 Wireframe-AI Automated Code Review" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

$issues = @()
$warnings = @()
$suggestions = @()

# Step 1: Check for uncommitted changes
Write-Host "`n📋 Step 1: Checking for uncommitted changes..." -ForegroundColor Yellow
$gitStatus = git status --porcelain 2>$null
if ($gitStatus) {
    Write-Host "⚠ Uncommitted changes detected" -ForegroundColor Yellow
    $warnings += "Commit changes before review for accurate results"
} else {
    Write-Host "✓ No uncommitted changes" -ForegroundColor Green
}

# Step 2: Get changed files
Write-Host "`n📋 Step 2: Identifying changed files..." -ForegroundColor Yellow
$changedFiles = git diff --name-only $branch 2>$null
if (-not $changedFiles) {
    Write-Host "ℹ No changes compared to $branch" -ForegroundColor Gray
    exit 0
}

Write-Host "Found $($changedFiles.Count) changed file(s)" -ForegroundColor Green

# Step 3: Run cargo check
Write-Host "`n📋 Step 3: Running cargo check..." -ForegroundColor Yellow
cargo check --quiet 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Cargo check failed" -ForegroundColor Red
    $issues += "Build fails - fix compilation errors"
} else {
    Write-Host "✓ Cargo check passed" -ForegroundColor Green
}

# Step 4: Run clippy
Write-Host "`n📋 Step 4: Running clippy..." -ForegroundColor Yellow
$clippyOutput = cargo clippy --quiet --message-format=short 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠ Clippy found issues" -ForegroundColor Yellow
    $warnings += "Clippy warnings found - review output"
    if ($Verbose) {
        Write-Host $clippyOutput -ForegroundColor Gray
    }
} else {
    Write-Host "✓ Clippy passed" -ForegroundColor Green
}

# Step 5: Check for common Wireframe-AI patterns
Write-Host "`n📋 Step 5: Checking Wireframe-AI patterns..." -ForegroundColor Yellow

$rustFiles = $changedFiles | Where-Object { $_ -match '\.rs$' }
foreach ($file in $rustFiles) {
    if (Test-Path $file) {
        $content = Get-Content $file -Raw
        
        # Check for TEAM_XXX comments
        if ($content -match 'TEAM_\d+') {
            Write-Host "✓ TEAM_XXX comments found in $file" -ForegroundColor Green
        } else {
            $suggestions += "Add TEAM_XXX comments to modified code in $file"
        }
        
        # Check for unsafe blocks
        if ($content -match 'unsafe\s*\{') {
            $warnings += "Unsafe block found in $file - requires security review"
        }
        
        # Check for TODO comments
        if ($content -match 'TODO') {
            $suggestions += "TODO comments found in $file - track in TODO.md"
        }
        
        # Check for unwrap() calls
        if ($content -match '\.unwrap\(\)') {
            $warnings += "unwrap() calls found in $file - consider proper error handling"
        }
        
        # Check for expect() calls
        if ($content -match '\.expect\(') {
            $warnings += "expect() calls found in $file - consider proper error handling"
        }
        
        # Check for hardcoded secrets
        if ($content -match '(password|secret|api_key|token)\s*=\s*"[^"]+"') {
            $issues += "Potential hardcoded secret in $file - remove or use environment variables"
        }
        
        # Check for NATS topic naming convention
        if ($content -match 'nats\.publish\(["\']([^"\']+)["\']') {
            $topic = $matches[1]
            if ($topic -notmatch '^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)*$') {
                $warnings += "NATS topic '$topic' in $file may not follow naming convention (lowercase, dot-separated)"
            }
        }
    }
}

# Step 6: Check for test coverage
Write-Host "`n📋 Step 6: Checking test coverage..." -ForegroundColor Yellow
$testFiles = $changedFiles | Where-Object { $_ -match 'test\.rs$|_test\.rs$|tests/' }
if (-not $testFiles -and $rustFiles) {
    $suggestions += "Consider adding tests for modified Rust files"
} else {
    Write-Host "✓ Test files found in changes" -ForegroundColor Green
}

# Step 7: Check documentation
Write-Host "`n📋 Step 7: Checking documentation..." -ForegroundColor Yellow
$docFiles = $changedFiles | Where-Object { $_ -match '\.md$' }
if (-not $docFiles -and $rustFiles) {
    $suggestions += "Consider updating documentation for modified code"
} else {
    Write-Host "✓ Documentation found in changes" -ForegroundColor Green
}

# Step 8: Check for schema changes
Write-Host "`n📋 Step 8: Checking for schema changes..." -ForegroundColor Yellow
$schemaFiles = $changedFiles | Where-Object { $_ -match 'schemas/' }
if ($schemaFiles) {
    $issues += "Schema changes detected - ensure backward compatibility and migration path"
    Write-Host "⚠ Schema changes found" -ForegroundColor Yellow
} else {
    Write-Host "✓ No schema changes" -ForegroundColor Green
}

# Step 9: Check for SDK changes
Write-Host "`n📋 Step 9: Checking for SDK changes..." -ForegroundColor Yellow
$sdkFiles = $changedFiles | Where-Object { $_ -match 'sdk/' }
if ($sdkFiles) {
    $warnings += "SDK changes detected - ensure public API compatibility and update documentation"
    Write-Host "⚠ SDK changes found" -ForegroundColor Yellow
} else {
    Write-Host "✓ No SDK changes" -ForegroundColor Green
}

# Step 10: Summary
Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "Code Review Summary" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

if ($issues.Count -gt 0) {
    Write-Host "`n🔴 Issues (must fix):" -ForegroundColor Red
    foreach ($issue in $issues) {
        Write-Host "  • $issue" -ForegroundColor Red
    }
}

if ($warnings.Count -gt 0) {
    Write-Host "`n🟡 Warnings (should review):" -ForegroundColor Yellow
    foreach ($warning in $warnings) {
        Write-Host "  • $warning" -ForegroundColor Yellow
    }
}

if ($suggestions.Count -gt 0) {
    Write-Host "`n🔵 Suggestions (optional improvements):" -ForegroundColor Blue
    foreach ($suggestion in $suggestions) {
        Write-Host "  • $suggestion" -ForegroundColor Blue
    }
}

if ($issues.Count -eq 0 -and $warnings.Count -eq 0 -and $suggestions.Count -eq 0) {
    Write-Host "`n✓ No issues found! Code looks good." -ForegroundColor Green
    exit 0
} elseif ($issues.Count -eq 0) {
    Write-Host "`n⚠ Review passed with warnings/suggestions" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "`n✗ Review failed with issues that must be fixed" -ForegroundColor Red
    exit 1
}
