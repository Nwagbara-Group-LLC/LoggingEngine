#!/usr/bin/env powershell
# Test runner script for LoggingEngine
# Usage: .\run_tests.ps1 [test_type] [options]

param(
    [Parameter(Position=0)]
    [ValidateSet("all", "unit", "integration", "benchmarks", "property", "e2e", "performance")]
    [string]$TestType = "all",
    
    [switch]$Release,
    [switch]$NoCoverage,
    [switch]$Verbose,
    [switch]$SkipSlowTests,
    [string]$Filter = "",
    [int]$Jobs = 4
)

$ErrorActionPreference = "Stop"

Write-Host "🚀 LoggingEngine Test Runner" -ForegroundColor Green
Write-Host "Test Type: $TestType" -ForegroundColor Cyan

# Set build profile
$BuildProfile = if ($Release) { "--release" } else { "" }
$ENV:RUST_LOG = if ($Verbose) { "debug" } else { "info" }

# Set test environment variables
$ENV:RUST_BACKTRACE = "1"
if ($SkipSlowTests) {
    $ENV:SKIP_SLOW_TESTS = "1"
}

# Create test output directory
$TestOutputDir = "target/test-results"
if (!(Test-Path $TestOutputDir)) {
    New-Item -ItemType Directory -Path $TestOutputDir -Force | Out-Null
}

function Run-UnitTests {
    Write-Host "`n📝 Running unit tests..." -ForegroundColor Yellow
    
    $cmd = "cargo test $BuildProfile --lib --bins"
    if ($Filter) { $cmd += " -- $Filter" }
    if ($Verbose) { $cmd += " --nocapture" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Unit tests failed"
    }
}

function Run-IntegrationTests {
    Write-Host "`n🔧 Running integration tests..." -ForegroundColor Yellow
    
    $cmd = "cargo test $BuildProfile --test integration"
    if ($Filter) { $cmd += " -- $Filter" }
    if ($Verbose) { $cmd += " --nocapture" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Integration tests failed"
    }
}

function Run-PropertyTests {
    Write-Host "`n🎲 Running property-based tests..." -ForegroundColor Yellow
    
    $cmd = "cargo test $BuildProfile --test property_tests"
    if ($Filter) { $cmd += " -- $Filter" }
    if ($Verbose) { $cmd += " --nocapture" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Property tests failed"
    }
}

function Run-Benchmarks {
    Write-Host "`n⚡ Running benchmarks..." -ForegroundColor Yellow
    
    $cmd = "cargo bench --bench ultra_logger_benchmarks"
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Benchmarks failed"
    }
    
    # Copy benchmark results
    if (Test-Path "target/criterion") {
        Copy-Item -Recurse "target/criterion" "$TestOutputDir/benchmark-results" -Force
        Write-Host "📊 Benchmark results saved to $TestOutputDir/benchmark-results" -ForegroundColor Green
    }
}

function Run-E2ETests {
    Write-Host "`n🌐 Running end-to-end tests..." -ForegroundColor Yellow
    
    $cmd = "cargo test $BuildProfile --test e2e"
    if ($Filter) { $cmd += " -- $Filter" }
    if ($Verbose) { $cmd += " --nocapture" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "E2E tests failed"
    }
}

function Run-PerformanceTests {
    Write-Host "`n🏎️ Running performance tests..." -ForegroundColor Yellow
    
    # Run specific performance-focused integration tests
    $cmd = "cargo test $BuildProfile test_ultra_low_latency test_high_throughput test_concurrent_logging test_end_to_end_trading_scenario"
    if ($Verbose) { $cmd += " --nocapture" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Performance tests failed"
    }
}

function Run-CodeCoverage {
    if ($NoCoverage) {
        Write-Host "`n⏭️ Skipping code coverage" -ForegroundColor Gray
        return
    }
    
    Write-Host "`n📊 Running code coverage analysis..." -ForegroundColor Yellow
    
    # Check if cargo-tarpaulin is installed
    $tarpaulinInstalled = cargo install --list | Select-String "cargo-tarpaulin"
    if (-not $tarpaulinInstalled) {
        Write-Host "Installing cargo-tarpaulin..." -ForegroundColor Gray
        cargo install cargo-tarpaulin
    }
    
    $cmd = "cargo tarpaulin --out Html --output-dir $TestOutputDir --timeout 300"
    if ($Filter) { $cmd += " --run-types Tests -- $Filter" }
    
    Write-Host "Command: $cmd" -ForegroundColor Gray
    Invoke-Expression $cmd
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "⚠️ Code coverage analysis failed, continuing..." -ForegroundColor Yellow
    } else {
        Write-Host "📊 Coverage report saved to $TestOutputDir/tarpaulin-report.html" -ForegroundColor Green
    }
}

function Show-TestSummary {
    Write-Host "`n📋 Test Summary" -ForegroundColor Green
    Write-Host "===============" -ForegroundColor Green
    
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "Test run completed at: $timestamp" -ForegroundColor White
    
    if (Test-Path "$TestOutputDir/tarpaulin-report.html") {
        Write-Host "📊 Coverage report: $TestOutputDir/tarpaulin-report.html" -ForegroundColor Cyan
    }
    
    if (Test-Path "$TestOutputDir/benchmark-results") {
        Write-Host "⚡ Benchmark results: $TestOutputDir/benchmark-results" -ForegroundColor Cyan
    }
    
    Write-Host "`n✅ All tests completed successfully!" -ForegroundColor Green
}

# Main execution
try {
    $startTime = Get-Date
    
    # Clean previous test results
    if (Test-Path $TestOutputDir) {
        Remove-Item -Recurse -Force $TestOutputDir
    }
    New-Item -ItemType Directory -Path $TestOutputDir -Force | Out-Null
    
    # Check Rust toolchain
    Write-Host "🦀 Checking Rust toolchain..." -ForegroundColor Gray
    cargo --version
    rustc --version
    
    # Build the project first
    Write-Host "`n🔨 Building project..." -ForegroundColor Yellow
    $buildCmd = "cargo build $BuildProfile --workspace"
    Write-Host "Command: $buildCmd" -ForegroundColor Gray
    Invoke-Expression $buildCmd
    
    if ($LASTEXITCODE -ne 0) {
        throw "Build failed"
    }
    
    # Run tests based on type
    switch ($TestType) {
        "all" {
            Run-UnitTests
            Run-IntegrationTests
            Run-PropertyTests
            Run-E2ETests
            Run-PerformanceTests
            Run-Benchmarks
        }
        "unit" { Run-UnitTests }
        "integration" { Run-IntegrationTests }
        "property" { Run-PropertyTests }
        "e2e" { Run-E2ETests }
        "performance" { Run-PerformanceTests }
        "benchmarks" { Run-Benchmarks }
    }
    
    # Run code coverage if requested
    if ($TestType -eq "all" -or $TestType -eq "unit" -or $TestType -eq "integration") {
        Run-CodeCoverage
    }
    
    $endTime = Get-Date
    $duration = $endTime - $startTime
    
    Write-Host "`n⏱️ Total test duration: $($duration.ToString('mm\:ss'))" -ForegroundColor Cyan
    
    Show-TestSummary
    
} catch {
    Write-Host "`n❌ Test execution failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}
