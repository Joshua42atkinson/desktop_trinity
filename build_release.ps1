# Build Frontend
Write-Host "Building Frontend..."
cd frontend
trunk build --release
if ($LASTEXITCODE -ne 0) { exit 1 }
cd ..

# Build Backend (which now embeds frontend)
Write-Host "Building Backend..."
cd backend
cargo build --release
if ($LASTEXITCODE -ne 0) { exit 1 }
cd ..

# Package
Write-Host "Packaging..."
if (-not (Test-Path "release")) { New-Item -ItemType Directory -Force -Path "release" }
Copy-Item "backend/target/release/backend.exe" "release/Daydream.exe"

Write-Host "Done! Executable is in release/Daydream.exe"
