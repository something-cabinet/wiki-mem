# WM Engine — Build & Serve Commands
# Install `just` via: cargo install just

# Build the web UI and Rust binary (release)
build-web:
    cd apps/wm-web && npx ng build
    cargo build --release --features "wm-server/web-ui"

# Development: Angular hot-reload + Rust API server in parallel
dev:
    @echo "Starting API server on :3000 and Angular dev server on :4200..."
    @echo "Open http://localhost:4200 in your browser."
    cargo run -- web --port 3000 &
    cd apps/wm-web && npx ng serve --proxy-config proxy.conf.json

# Serve the API server (web UI served from embedded assets or disk)
serve port="3000":
    cargo run -- web --port {{port}}

# Build everything (debug mode)
build:
    cd apps/wm-web && npx ng build
    cargo build --features "wm-server/web-ui"

# Run tests
test:
    cargo test --workspace
