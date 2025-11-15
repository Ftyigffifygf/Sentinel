# Verification script for Rust workspace structure

Write-Host "Verifying Rust workspace structure..." -ForegroundColor Green

# Check root Cargo.toml
if (Test-Path "Cargo.toml") {
    Write-Host "✓ Root Cargo.toml exists" -ForegroundColor Green
} else {
    Write-Host "✗ Root Cargo.toml missing" -ForegroundColor Red
}

# Check all crates
$crates = @(
    "api-layer",
    "static-worker", 
    "dynamic-worker",
    "behavioral-worker",
    "shared-domain",
    "shared-crypto",
    "shared-db"
)

foreach ($crate in $crates) {
    $cratePath = "crates/$crate"
    $cargoToml = "$cratePath/Cargo.toml"
    $srcLib = "$cratePath/src/lib.rs"
    $srcMain = "$cratePath/src/main.rs"
    
    Write-Host ""
    Write-Host "Checking $crate..." -ForegroundColor Cyan
    
    if (Test-Path $cargoToml) {
        Write-Host "  ✓ Cargo.toml exists" -ForegroundColor Green
    } else {
        Write-Host "  ✗ Cargo.toml missing" -ForegroundColor Red
    }
    
    if (Test-Path $srcLib) {
        Write-Host "  ✓ src/lib.rs exists" -ForegroundColor Green
    }
    
    if (Test-Path $srcMain) {
        Write-Host "  ✓ src/main.rs exists" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Workspace structure verification complete!" -ForegroundColor Green
Write-Host ""
Write-Host "To build the project (requires Rust):" -ForegroundColor Yellow
Write-Host "  cargo build --workspace" -ForegroundColor White
