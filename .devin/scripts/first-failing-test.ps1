# First Failing Test Extractor for Wireframe-AI
# Extracts the first failing test with detailed error information
# Helps agents focus on specific test failures rather than running the entire suite

param(
    [string]$TestCommand = "cargo test",
    [switch]$Verbose
)

Write-Host "🧪 Running tests to find first failure..." -ForegroundColor Cyan

# Run tests and capture output
$output = Invoke-Expression $TestCommand 2>&1
$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    Write-Host "✓ All tests passed!" -ForegroundColor Green
    exit 0
}

Write-Host "✗ Tests failed. Extracting first failure..." -ForegroundColor Yellow

# Parse output to find first failing test
$lines = $output -split "`n"
$firstFailure = $null
$failureDetails = @()
$inFailure = $false
$testName = $null

for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    
    # Look for test failure indicators
    if ($line -match "test\s+(.+?)\s+\.\.\.\s+FAILED") {
        $testName = $matches[1]
        $firstFailure = $line
        $inFailure = $true
        $failureDetails += $line
        break
    }
    
    if ($line -match "test\s+(.+?)\s+\.\.\.\s+FAILED") {
        $testName = $matches[1]
        $firstFailure = $line
        $inFailure = $true
        $failureDetails += $line
        break
    }
}

# If we found a failure, collect details
if ($firstFailure) {
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "First Failing Test" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host $firstFailure -ForegroundColor Red
    
    # Collect error details (next 20 lines or until next test)
    $detailCount = 0
    for ($j = $i + 1; $j -lt $lines.Count -and $detailCount -lt 20; $j++) {
        $detailLine = $lines[$j]
        
        # Stop if we hit another test
        if ($detailLine -match "^test\s+") {
            break
        }
        
        $failureDetails += $detailLine
        $detailCount++
    }
    
    Write-Host "`nError Details:" -ForegroundColor Yellow
    foreach ($detail in $failureDetails) {
        Write-Host $detail -ForegroundColor Gray
    }
    
    Write-Host "`n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "Suggested Command:" -ForegroundColor Cyan
    Write-Host "cargo test $testName" -ForegroundColor Green
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    
    if ($Verbose) {
        Write-Host "`nFull test output:" -ForegroundColor Yellow
        $output
    }
    
    exit 1
} else {
    Write-Host "⚠ Could not identify specific failing test" -ForegroundColor Yellow
    Write-Host "Full output:" -ForegroundColor Yellow
    $output
    exit 1
}
