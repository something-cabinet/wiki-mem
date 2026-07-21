# WM Engine — Build & Serve Commands
# Install `just` via: cargo install just
# Install `oxmgr` via: brew install oxmgr

# ─── Dev Server (oxmgr managed) ───────────────────────

# Start all dev processes (wm-server + Angular)
dev:
    oxmgr apply oxfile.toml

# Start wm-server only
server:
    oxmgr start server

# Start Angular dev server only
web:
    oxmgr start web

# Stop all dev processes
stop:
    oxmgr stop server web

# Restart server (after code changes)
restart:
    oxmgr restart server

# View logs
logs:
    oxmgr logs -f server

# ─── Production / Build ───────────────────────────────

# Build Angular + Rust server
build:
    cd apps/wm-web && npx ng build
    cargo build -p wm-server

# Start the server daemon (:4090)
serve port="4090":
    cargo run -p wm-server -- --port {{port}}

# Start MCP proxy (connects to running wm-server via oxmgr)
mcp:
    cargo run -p wm-cli mcp

# ─── E2E Tests ─────────────────────────────────────────

# Run E2E tests (requires: Terminal 1: `just dev`, Terminal 2: `npm run test:e2e`)
e2e:
    cd apps/wm-web-e2e && npm run test:e2e

# Start mock server + Angular with E2E proxy (via oxmgr), then run tests
e2e-dev:
    source ~/.nvm/nvm.sh && nvm use 24.15.0 2>/dev/null
    oxmgr apply oxfile.toml --only mock,web-e2e
    sleep 8
    cd apps/wm-web-e2e && npm run test:e2e; EXIT_CODE=$$?
    oxmgr stop mock web-e2e 2>/dev/null
    exit $$EXIT_CODE

# Start mock server only (for Terminal 1 in manual E2E workflow)
e2e-mock:
    bun packages/wm-mock-server/src/bun-entry.ts --mappings apps/wm-web-e2e/mappings --port 8081

# Start Angular with E2E proxy (for Terminal 2 in manual E2E workflow)
e2e-web:
    cd apps/wm-web && npx ng serve --proxy-config ../wm-web-e2e/proxy.e2e.conf.json

# ─── Testing ──────────────────────────────────────────

# Run all workspace tests
test:
    cargo test --workspace

# Run just CLI/MCP tests (faster)
test-cli:
    cargo test -p wm-core --test cli_test
    cargo test -p wm-core --test mcp_test

# ─── Utility ──────────────────────────────────────────

# Check only one .wm/ directory exists
check-wm-dirs:
    count=$(find . -name ".wm" -type d -not -path "./.wm" | wc -l)
    if [ "$count" -gt 0 ]; then echo "Found $count rogue .wm/ director(ies)"; exit 1; fi
    echo "OK: Only one .wm/ at project root."

# Full CI pipeline
ci: test check-wm-dirs

# Show all available commands
default:
    just --list
