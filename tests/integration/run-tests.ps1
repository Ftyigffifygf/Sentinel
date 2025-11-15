# PowerShell script to run integration tests
# Usage: .\run-tests.ps1 [test-suite]

param(
    [string]$TestSuite = "all",
    [switch]$Verbose,
    [switch]$SkipSetup
)

$ErrorActionPreference = "Stop"

Write-Host "=== Security SaaS Platform - Integration Tests ===" -ForegroundColor Cyan
Write-Host ""

# Check if services are running
function Test-ServicesRunning {
    Write-Host "Checking if services are running..." -ForegroundColor Yellow
    
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080/health" -TimeoutSec 5 -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            Write-Host "✓ API Layer is running" -ForegroundColor Green
            return $true
        }
    }
    catch {
        Write-Host "✗ API Layer is not responding" -ForegroundColor Red
        Write-Host "  Please start services with: docker-compose up -d" -ForegroundColor Yellow
        return $false
    }
}

# Setup environment
function Initialize-TestEnvironment {
    Write-Host "Setting up test environment..." -ForegroundColor Yellow
    
    # Set environment variables
    $env:DATABASE_URL = "postgres://postgres:postgres@localhost:5432/security_saas"
    $env:API_BASE_URL = "http://localhost:8080"
    $env:WS_BASE_URL = "ws://localhost:8080"
    
    Write-Host "✓ Environment variables set" -ForegroundColor Green
}

# Run specific test suite
function Invoke-TestSuite {
    param([string]$Suite)
    
    $testArgs = @("test", "--package", "integration-tests")
    
    if ($Suite -ne "all") {
        $testArgs += @("--test", $Suite)
    }
    
    $testArgs += @("--", "--ignored")
    
    if ($Verbose) {
        $testArgs += "--nocapture"
    }
    
    Write-Host ""
    Write-Host "Running test suite: $Suite" -ForegroundColor Cyan
    Write-Host "Command: cargo $($testArgs -join ' ')" -ForegroundColor Gray
    Write-Host ""
    
    & cargo $testArgs
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "✅ Test suite '$Suite' PASSED" -ForegroundColor Green
        return $true
    }
    else {
        Write-Host ""
        Write-Host "❌ Test suite '$Suite' FAILED" -ForegroundColor Red
        return $false
    }
}

# Main execution
try {
    if (-not $SkipSetup) {
        if (-not (Test-ServicesRunning)) {
            Write-Host ""
            Write-Host "Services are not running. Please start them first:" -ForegroundColor Yellow
            Write-Host "  docker-compose up -d" -ForegroundColor White
            Write-Host "  OR" -ForegroundColor White
            Write-Host "  kubectl apply -f k8s/" -ForegroundColor White
            exit 1
        }
        
        Initialize-TestEnvironment
    }
    
    Write-Host ""
    Write-Host "Starting integration tests..." -ForegroundColor Cyan
    Write-Host ""
    
    $success = $true
    
    switch ($TestSuite.ToLower()) {
        "all" {
            Write-Host "Running all test suites..." -ForegroundColor Yellow
            Write-Host ""
            
            $suites = @(
                "complete_analysis_flow",
                "multi_tenant_isolation",
                "siem_integration",
                "endpoint_monitoring",
                "load_testing",
                "security_testing"
            )
            
            foreach ($suite in $suites) {
                if (-not (Invoke-TestSuite -Suite $suite)) {
                    $success = $false
                }
                Write-Host ""
            }
        }
        "analysis" { $success = Invoke-TestSuite -Suite "complete_analysis_flow" }
        "tenant" { $success = Invoke-TestSuite -Suite "multi_tenant_isolation" }
        "siem" { $success = Invoke-TestSuite -Suite "siem_integration" }
        "endpoint" { $success = Invoke-TestSuite -Suite "endpoint_monitoring" }
        "load" { $success = Invoke-TestSuite -Suite "load_testing" }
        "security" { $success = Invoke-TestSuite -Suite "security_testing" }
        default {
            Write-Host "Unknown test suite: $TestSuite" -ForegroundColor Red
            Write-Host ""
            Write-Host "Available test suites:" -ForegroundColor Yellow
            Write-Host "  all       - Run all test suites" -ForegroundColor White
            Write-Host "  analysis  - Complete analysis flow tests" -ForegroundColor White
            Write-Host "  tenant    - Multi-tenant isolation tests" -ForegroundColor White
            Write-Host "  siem      - SIEM integration tests" -ForegroundColor White
            Write-Host "  endpoint  - Endpoint monitoring tests" -ForegroundColor White
            Write-Host "  load      - Load testing" -ForegroundColor White
            Write-Host "  security  - Security testing" -ForegroundColor White
            exit 1
        }
    }
    
    Write-Host ""
    Write-Host "=== Test Execution Complete ===" -ForegroundColor Cyan
    
    if ($success) {
        Write-Host "✅ All tests PASSED" -ForegroundColor Green
        exit 0
    }
    else {
        Write-Host "❌ Some tests FAILED" -ForegroundColor Red
        exit 1
    }
}
catch {
    Write-Host ""
    Write-Host "Error running tests: $_" -ForegroundColor Red
    exit 1
}
