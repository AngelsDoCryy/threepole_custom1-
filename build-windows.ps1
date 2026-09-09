$ErrorActionPreference = "Stop"

if (-not $env:BUNGIE_API_KEY) {
    Write-Host "BUNGIE_API_KEY is not set." -ForegroundColor Red
    Write-Host 'Set it for this PowerShell session with:'
    Write-Host '$env:BUNGIE_API_KEY="YOUR_BUNGIE_API_KEY"'
    exit 1
}

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    throw "Node.js was not found. Install Node.js first."
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Rust/Cargo was not found. Install Rust with rustup first."
}

if (-not (Get-Command yarn -ErrorAction SilentlyContinue)) {
    if (Get-Command corepack -ErrorAction SilentlyContinue) {
        corepack enable
        corepack prepare yarn@1.22.22 --activate
    } else {
        npm install -g yarn
    }
}

yarn install
yarn tauri build

Write-Host "Build finished. Check src-tauri\target\release\bundle\msi\" -ForegroundColor Green
