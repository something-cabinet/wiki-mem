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

# Check that only one .wm/ directory exists (at project root)
check-wm-dirs:
    $count = (Get-ChildItem -Recurse -Directory -Filter ".wm" | Where-Object { $_.FullName -ne (Join-Path (Get-Location) ".wm") }).Count
    if ($count -gt 0) { throw "Found $count rogue .wm/ director(ies)" }
    else { Write-Host "OK: Only one .wm/ at project root." }

# Full CI pipeline
ci: test check-wm-dirs

# Build the Tauri desktop app (Angular + Rust) — does NOT start it
tauri-build:
    Set-Location apps/wm-web; npx ng build
    Set-Location apps/wm-web/src-tauri; cargo build

# Start the pre-built Tauri binary in background (for tauri-pilot testing)
# Use this when you want to test remotely — binary runs, tauri-pilot connects.
tauri-run:
    Write-Host "Starting wm-tauri.exe in background..."
    Write-Host "Connect with: tauri-pilot ping"
    cmd.exe /c start "WM Tauri" "target\debug\wm-tauri.exe"

# Build + start (convenience)
tauri-dev: tauri-build tauri-run

# Tauri dev with hot-reload (Angular + Tauri) — opens a window
# Use this when you're at the machine and want to edit code
tauri-watch:
    Set-Location apps/wm-web; npm run tauri
