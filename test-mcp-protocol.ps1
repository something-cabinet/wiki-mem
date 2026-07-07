# ─── MCP Protocol Test: Knowns vs WM ───
# Tests the actual JSON-RPC MCP handshake and tool discovery

$wm = "C:\Users\hk\.kimaki\projects\vpp-rag\target\debug\wm-cli.exe"
$knowns = "C:\Users\hk\.knowns\bin\knowns.exe"
$tmp = Join-Path $env:TEMP "mcp-test-$(Get-Random)"

Write-Host "=== MCP Protocol Test: Knowns vs WM ===" -ForegroundColor Cyan
Write-Host ""

# ─── Helper: Test an MCP server ───
function Test-McpServer {
    param($Name, $Command, $Args, $ProjectDir)

    Write-Host "─── Testing $Name ───" -ForegroundColor Yellow

    # Initialize test project
    if ($ProjectDir) {
        New-Item -ItemType Directory -Path $ProjectDir -Force | Out-Null
        Push-Location $ProjectDir
    }

    # Start the server process
    $proc = Start-Process -FilePath $Command -ArgumentList $Args -NoNewWindow -PassThru -RedirectStandardInput "nul" -RedirectStandardOutput "$tmp\$Name-out.txt" -RedirectStandardError "$tmp\$Name-err.txt"

    # Actually, Start-Process with stdin piping is tricky in PowerShell.
    # Let's use a different approach: write requests to a file, pipe them.
    
    Pop-Location
}

# ─── Simpler approach: test via pipe ───
Write-Host "1. Testing WM MCP handshake..." -ForegroundColor Green

# WM test project
$wmDir = Join-Path $tmp "wm"
New-Item -ItemType Directory -Path $wmDir -Force | Out-Null

# Create a test page for WM
@"
---
title: Test
type: concept
---
# Test
"@ | Set-Content "$wmDir\.wm\wiki\concepts\test.md" -ErrorAction SilentlyContinue

# Build the init + search requests
$wmRequests = @'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1"}},"id":1}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}
'@

Write-Host "  Sending init + tools/list to WM..."
$wmRequests | & $wm serve 2>"$tmp\wm-stderr.txt" | Select-Object -First 3 > "$tmp\wm-responses.txt"
Write-Host "  WM responses captured."

# Parse JSON responses
$wmLines = Get-Content "$tmp\wm-responses.txt"
foreach ($line in $wmLines) {
    if ($line -match '"jsonrpc"') {
        try {
            $obj = $line | ConvertFrom-Json
            if ($obj.id -eq 2 -and $obj.result) {
                $toolCount = @($obj.result).Count
                Write-Host "  WM tools/list: $toolCount tools registered" -ForegroundColor Green
                
                # Show first 5 tools
                $obj.result | Select-Object -First 5 | ForEach-Object {
                    Write-Host "    - $($_.name)"
                }
                
                # Count unique tool groups
                $groups = @($obj.result | ForEach-Object { 
                    if ($_.name -match '^wm_(\w+)') { $matches[1] } 
                } | Sort-Object -Unique)
                Write-Host "  WM tool groups: $($groups -join ', ')" -ForegroundColor Green
            }
        } catch {
            # Partial response or non-JSON
        }
    }
}

# Count total tools from wm_initial
$wmInitReq = '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"wm_initial","arguments":{}},"id":3}'
$wmInitReq | & $wm serve 2>"$tmp\wm-stderr2.txt" | Select-Object -First 1 > "$tmp\wm-init.txt"
$initLine = Get-Content "$tmp\wm-init.txt" -ErrorAction SilentlyContinue
if ($initLine) {
    try {
        $init = $initLine | ConvertFrom-Json
        if ($init.result) {
            Write-Host "`n  wm_initial response:" -ForegroundColor Green
            Write-Host "    graph_nodes: $($init.result.graph_nodes)"
            Write-Host "    graph_edges: $($init.result.graph_edges)"
            Write-Host "    search_modes: $($init.result.search_modes_available -join ', ')"
        }
    } catch {
        Write-Host "  wm_initial parse failed: $_" -ForegroundColor Red
    }
}

# ─── Now test Knowns ───
Write-Host "`n2. Testing Knowns MCP handshake..." -ForegroundColor Green

$knownsDir = Join-Path $tmp "knowns"
New-Item -ItemType Directory -Path $knownsDir -Force | Out-Null

$knownsRequests = @'
{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1"}},"id":1}
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}
'@

Write-Host "  Sending init + tools/list to Knowns..."
try {
    $knownsRequests | & $knowns mcp --stdio 2>"$tmp\knowns-stderr.txt" | Select-Object -First 3 > "$tmp\knowns-responses.txt" 2>$null
    Write-Host "  Knowns responses captured."
    
    $knownsLines = Get-Content "$tmp\knowns-responses.txt" -ErrorAction SilentlyContinue
    foreach ($line in $knownsLines) {
        if ($line -match '"jsonrpc"') {
            try {
                $obj = $line | ConvertFrom-Json
                if ($obj.id -eq 2 -and $obj.result) {
                    $toolCount = @($obj.result).Count
                    Write-Host "  Knowns tools/list: $toolCount tools registered" -ForegroundColor Green
                    
                    # Show first 5 tools
                    $obj.result | Select-Object -First 5 | ForEach-Object {
                        Write-Host "    - $($_.name)"
                    }
                }
                if ($obj.id -eq 1 -and $obj.result) {
                    Write-Host "  Knowns server: $($obj.result.serverInfo.name) v$($obj.result.serverInfo.version)" -ForegroundColor Green
                }
            } catch {
                # Partial response
            }
        }
    }
} catch {
    Write-Host "  Knowns MCP test failed: $_" -ForegroundColor Red
}

# ─── Summary ───
Write-Host "`n=== SUMMARY ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Both servers tested via MCP JSON-RPC 2.0 over stdio." -ForegroundColor White
Write-Host ""
Write-Host "To use both in OpenCode:" -ForegroundColor Yellow
Write-Host "  opencode.json already has both configured." -ForegroundColor White
Write-Host "  The agent discovers both tool sets and chooses based on context:" -ForegroundColor White
Write-Host "  - Knowns: doc/task/memory management" -ForegroundColor White
Write-Host "  - WM:     typed graph, time tracking, semantic search" -ForegroundColor White
Write-Host ""
Write-Host "Test artifacts: $tmp" -ForegroundColor DarkGray

# Cleanup
Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
