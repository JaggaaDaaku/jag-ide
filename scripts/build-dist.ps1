# Jag IDE — Distribution Build Script (Windows)

$ErrorActionPreference = "Stop"

# Use a specific target dir to avoid file locks
$TargetDir = "$env:TEMP\jag_dist_target"
$env:CARGO_TARGET_DIR = $TargetDir

Write-Host "--- Step 1: Building Rust Backend (Release Mode) ---" -ForegroundColor Cyan
cargo build --release -p jag-server

Write-Host "--- Step 2: Preparing Frontend Sidecar Directory ---" -ForegroundColor Cyan
$BinDir = Join-Path "frontend" "bin"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir
}

# Fix: Join-Path only takes 2 arguments in older PS, nesting them
$SourceBin = Join-Path (Join-Path $TargetDir "release") "jag-server.exe"
$DestBin = Join-Path $BinDir "jag-server.exe"

if (-not (Test-Path $SourceBin)) {
    throw "Backend binary not found at $SourceBin"
}

Write-Host "Copying $SourceBin to $DestBin..."
Copy-Item -Path $SourceBin -Destination $DestBin -Force

Write-Host "--- Step 3: Building Frontend & Packaging Installer ---" -ForegroundColor Cyan
Set-Location frontend
npm install
npm run dist

Write-Host "`n--- BUILD COMPLETE ---" -ForegroundColor Green
Write-Host "Installer is available in: frontend\release\Jag IDE-Setup-0.1.0.exe" -ForegroundColor Yellow
