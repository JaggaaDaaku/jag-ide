# Jag IDE Production Build Script
param(
    [switch]$DryRun
)

Write-Host "Starting Jag IDE Production Build..." -ForegroundColor Cyan
if ($DryRun) { Write-Host "[DRY RUN MODE] - No changes will be made." -ForegroundColor Magenta }

# 1. Build Rust Backend
Write-Host "Step 1: Building Rust Backend (Release)..." -ForegroundColor Yellow
if (-not $DryRun) {
    cargo build -p jag-server --release
    if ($LASTEXITCODE -ne 0) { Write-Error "Backend build failed"; exit 1 }
}

# 2. Ensure bin directory exists in frontend
$binDir = "frontend/bin"
if (!(Test-Path $binDir)) {
    if (-not $DryRun) { New-Item -ItemType Directory -Path $binDir }
}

# 3. Copy binary to frontend
Write-Host "Step 2: Bundling Backend Sidecar..." -ForegroundColor Yellow
if (-not $DryRun) {
    Copy-Item "target/release/jag-server.exe" "frontend/bin/jag-server.exe" -Force
}

# 4. Build Frontend
Write-Host "Step 3: Building Frontend..." -ForegroundColor Yellow
if (-not $DryRun) {
    Set-Location frontend
    npm install
    npm run build
    if ($LASTEXITCODE -ne 0) { Write-Error "Frontend build failed"; exit 1 }
    Set-Location ..
}

# 5. Generate Installer
Write-Host "Step 4: Generating Installer (Electron Builder)..." -ForegroundColor Yellow
if (-not $DryRun) {
    Set-Location frontend
    npm run dist
    if ($LASTEXITCODE -ne 0) { Write-Error "Installer generation failed"; exit 1 }
    Set-Location ..
}

if (-not $DryRun) {
    Write-Host "Step 5: Generating Checksums..." -ForegroundColor Yellow
    $installer = Get-ChildItem "frontend/release/*.exe" | Select-Object -First 1
    if ($installer) {
        $hash = Get-FileHash $installer.FullName -Algorithm SHA256
        $hash.Hash | Out-File "frontend/release/checksums.txt"
        Write-Host "SHA256: $($hash.Hash)" -ForegroundColor Gray
    }
    Write-Host "Build Complete! Check frontend/release for artifacts." -ForegroundColor Green
} else {
    Write-Host "Dry run complete. Paths validated." -ForegroundColor Green
}
