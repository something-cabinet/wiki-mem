# WM Engine — Build & Serve Commands
# Install `just` via: cargo install just

set shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-NonInteractive", "-Command"]

# Build the web UI and Rust binary (release)
build-web:
    Set-Location apps/wm-web; npx ng build
    cargo build --release --features "wm-server/web-ui"

# Development: Angular hot-reload + Rust API server in parallel
dev:
    Write-Host "Starting API server on :3000 and Angular dev server on :4200..."
    Write-Host "Open http://localhost:4200 in your browser."
    cargo run -- web --port 3000 &
    Set-Location apps/wm-web; npx ng serve --proxy-config proxy.conf.json

# Serve the API server (web UI served from embedded assets or disk)
serve port="3000":
    cargo run -- web --port {{port}}

# Build everything (debug mode)
build:
    Set-Location apps/wm-web; npx ng build
    cargo build --features "wm-server/web-ui"

# Run tests
test:
    cargo test --workspace

# Build the Tauri desktop app (Angular + Rust)
tauri-build:
    Set-Location apps/wm-web; npx ng build
    Set-Location apps/wm-web/src-tauri; cargo build

# Start Tauri desktop app (builds first if needed)
tauri-dev: tauri-build
    Write-Host "Starting Tauri desktop app..."
    Write-Host "Connect with: tauri-pilot ping"
    Start-Process -NoNewWindow -FilePath "target\debug\wm-tauri.exe" -WorkingDirectory (Get-Location)

# Tauri dev with hot-reload (Angular + Tauri)
tauri-watch:
    Set-Location apps/wm-web; npm run tauri
