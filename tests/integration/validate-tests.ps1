# Validation script for integration tests
# Checks that all test files are properly structured

$ErrorActionPreference = "Stop"

Write-Host "=== Integration Tests Validation ===" -ForegroundColor Cyan
Write-Host ""

$testFiles = @(
    "tests/complete_analysis_flow.rs",
    "tests/multi_tenant_isolation.rs",
    "tests/siem_integration.rs",
    "tests/endpoint_monitoring.rs",
    "tests/load_testing.rs",
    "tests/security_testing.rs"
)

$allValid = $true

foreach ($file in $testFiles) {
    $fullPath = Join-Path $PSScriptRoot $file
    
    if (Test-Path $fullPath) {
        $content = Get-Content $fullPath -Raw
        
        # Check for required elements
        $hasTests = $content -match '#\[tokio::test\]'
        $hasIgnore = $content -match '#\[ignore\]'
        $hasRequirements = $content -match 'Requirements:'
        
        if ($hasTests -and $hasIgnore -and $hasRequirements) {
            Write-Host "✓ $file" -ForegroundColor Green
        }
        else {
            Write-Host "✗ $file - Missing required elements" -ForegroundColor Red
            if (-not $hasTests) { Write-Host "  - Missing #[tokio::test]" -ForegroundColor Yellow }
            if (-not $hasIgnore) { Write-Host "  - Missing #[ignore]" -ForegroundColor Yellow }
            if (-not $hasRequirements) { Write-Host "  - Missing Requirements comment" -ForegroundColor Yellow }
            $allValid = $false
        }
    }
    else {
        Write-Host "✗ $file - File not found" -ForegroundColor Red
        $allValid = $false
    }
}

Write-Host ""

# Check Cargo.toml
$cargoPath = Join-Path $PSScriptRoot "Cargo.toml"
if (Test-Path $cargoPath) {
    Write-Host "✓ Cargo.toml exists" -ForegroundColor Green
}
else {
    Write-Host "✗ Cargo.toml not found" -ForegroundColor Red
    $allValid = $false
}

# Check README
$readmePath = Join-Path $PSScriptRoot "README.md"
if (Test-Path $readmePath) {
    Write-Host "✓ README.md exists" -ForegroundColor Green
}
else {
    Write-Host "✗ README.md not found" -ForegroundColor Red
    $allValid = $false
}

Write-Host ""

if ($allValid) {
    Write-Host "✅ All validation checks passed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To compile and run tests:" -ForegroundColor Cyan
    Write-Host "  cargo check --package integration-tests" -ForegroundColor White
    Write-Host "  cargo test --package integration-tests -- --ignored" -ForegroundColor White
    exit 0
}
else {
    Write-Host "❌ Some validation checks failed" -ForegroundColor Red
    exit 1
}
