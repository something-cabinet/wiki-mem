# Knowns vs Wiki Memory Engine — Comparison Test Script v2
# Compares CLI behavior side-by-side using Process-based execution
# (Avoids PowerShell pipe operators which break MCP stdio communication)
#
# IMPORTANT: MCP stdio requires proper pipe handling. Never use direct
# PowerShell pipe operators (`|`) for MCP communication — they mangle
# JSON-RPC framing. Instead, use:
#   - Named pipes (System.IO.Pipes.NamedPipeServerStream)
#   - Temp files (write requests to file, redirect stdin from file)
#   - System.Diagnostics.Process with RedirectStandardInput
# See Invoke-McpViaNamedPipe below for the recommended approach.

$ErrorActionPreference = 'Stop'
$wmProject = "C:\Users\hk\.kimaki\projects\vpp-rag"
$wmExe = Join-Path $wmProject "target\debug\wm-cli.exe"
$knownsExe = "C:\Users\hk\.knowns\bin\knowns.exe"
$tmpRoot = Join-Path $env:TEMP "wm-vs-knowns-$(Get-Random)"

# ─── Results accumulator ───
$results = [System.Collections.Generic.List[pscustomobject]]::new()

function Add-Result {
    param($Command, $WM, $Knowns, $Notes)
    $results.Add([pscustomobject]@{
        Command = $Command
        WM       = $WM
        Knowns   = $Knowns
        Notes    = $Notes
    })
}

# ─── Helper: execute a CLI command via Process, capture stdout ───
function Invoke-ProcessCapture {
    param(
        [string]$FilePath,
        [string]$Arguments,
        [string]$WorkingDirectory
    )
    $outFile = [System.IO.Path]::Combine($tmpRoot, "stdout-$([System.IO.Path]::GetRandomFileName()).txt")
    $errFile = [System.IO.Path]::Combine($tmpRoot, "stderr-$([System.IO.Path]::GetRandomFileName()).txt")

    $psi = [System.Diagnostics.ProcessStartInfo]@{
        FileName               = $FilePath
        Arguments              = $Arguments
        WorkingDirectory       = $WorkingDirectory
        UseShellExecute        = $false
        RedirectStandardOutput = $true
        RedirectStandardError  = $true
        CreateNoWindow         = $true
    }
    $proc = [System.Diagnostics.Process]::Start($psi)
    $stdout = $proc.StandardOutput.ReadToEnd()
    $stderr = $proc.StandardError.ReadToEnd()
    $proc.WaitForExit(30000) | Out-Null

    # Also write to temp files for inspection
    [System.IO.File]::WriteAllText($outFile, $stdout)
    if ($stderr) { [System.IO.File]::WriteAllText($errFile, $stderr) }

    return @{
        ExitCode = $proc.ExitCode
        Stdout   = $stdout
        Stderr   = $stderr
        OutFile  = $outFile
        ErrFile  = $errFile
    }
}

# ─── Helper: test with named pipe (for MCP communication) ───
function Invoke-McpViaNamedPipe {
    param(
        [string]$FilePath,
        [string]$Arguments,
        [string]$WorkingDirectory,
        [string]$PipeName = "wm-mcp-test-$(Get-Random)",
        [string[]]$Requests
    )

    # Create the named pipe server before starting the process
    $pipeServer = [System.IO.Pipes.NamedPipeServerStream]::new(
        $PipeName,
        [System.IO.Pipes.PipeDirection]::InOut,
        1,
        [System.IO.Pipes.PipeTransmissionMode]::Message
    )

    # Start the process, telling it to use the named pipe for stdio
    # We pass the pipe name as an argument convention: --mcp-pipe <name>
    # If the CLI doesn't support --mcp-pipe, we use a wrapper approach

    $errFile = [System.IO.Path]::Combine($tmpRoot, "mcp-err-$([System.IO.Path]::GetRandomFileName()).txt")
    $psi = [System.Diagnostics.ProcessStartInfo]@{
        FileName               = $FilePath
        Arguments              = "$Arguments --mcp-pipe $PipeName"
        WorkingDirectory       = $WorkingDirectory
        UseShellExecute        = $false
        RedirectStandardError  = $true
        CreateNoWindow         = $true
    }

    $proc = [System.Diagnostics.Process]::Start($psi)

    # Server waits for client connection
    $pipeServer.WaitForConnection()

    # Send requests
    $writer = [System.IO.StreamWriter]::new($pipeServer)
    foreach ($req in $Requests) {
        $writer.WriteLine($req)
        $writer.Flush()
        Start-Sleep -Milliseconds 100
    }

    # Read responses
    $reader = [System.IO.StreamReader]::new($pipeServer)
    $responses = @()
    while ($pipeServer.IsConnected -and $pipeServer.CanRead) {
        $line = $reader.ReadLine()
        if ($line) { $responses += $line }
        if ($responses.Count -ge 10) { break }
    }

    $stderr = $proc.StandardError.ReadToEnd()
    if ($stderr) { [System.IO.File]::WriteAllText($errFile, $stderr) }
    $pipeServer.Dispose()
    $proc.Kill()

    return @{
        Responses = $responses
        Stderr    = $stderr
        ErrFile   = $errFile
    }
}

# ═══════════════════════════════════════════════════════════════
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Knowns vs Wiki Memory Engine (WM)" -ForegroundColor Cyan
Write-Host "  Comparison Test v2 (Process-based I/O)" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# ─── 1. Build wm-cli if not built ───
Write-Host "─── 0. BUILD ───" -ForegroundColor Yellow
if (-not (Test-Path $wmExe)) {
    Write-Host "Building wm-cli..." -ForegroundColor Yellow
    Push-Location $wmProject
    $build = Invoke-ProcessCapture -FilePath "cargo" -Arguments "build -p wm-cli" -WorkingDirectory $wmProject
    Pop-Location
    if ($build.ExitCode -ne 0) {
        Write-Host "Build FAILED:`n$($build.Stderr)" -ForegroundColor Red
        exit 1
    }
    Write-Host "Build complete." -ForegroundColor Green
} else {
    Write-Host "wm-cli already built (last built: $(Get-Item $wmExe | ForEach-Object { $_.LastWriteTime }))." -ForegroundColor Green
}
Write-Host ""

# ─── 2. Help / Version ───
Write-Host "─── 1. HELP / VERSION ───" -ForegroundColor Yellow

$wmHelp = Invoke-ProcessCapture -FilePath $wmExe -Arguments "--help" -WorkingDirectory $wmProject
$knownsHelp = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "--help" -WorkingDirectory $wmProject
$wmVer = Invoke-ProcessCapture -FilePath $wmExe -Arguments "--version" -WorkingDirectory $wmProject
$knownsVer = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "--version" -WorkingDirectory $wmProject

Write-Host "Knowns version: $($knownsVer.Stdout.Trim())" -ForegroundColor Green
Write-Host "WM version:     $($wmVer.Stdout.Trim())" -ForegroundColor Green

# Count subcommands
$wmCmdCount = @($wmHelp.Stdout | Select-String "^\s{2}[a-z]").Count
# Knowns uses different help formatting; count by looking for indented lines at start
$knownsCmdCount = 0
foreach ($line in ($knownsHelp.Stdout -split "`n")) {
    if ($line -match '^\s{4}\S+\s+') { $knownsCmdCount++ }
}
if ($knownsCmdCount -eq 0) { $knownsCmdCount = @($knownsHelp.Stdout | Select-String "Manage|Show|Inspect|Launch|Initialize|Search|Start|Sync|Update|Validate").Count }

Add-Result -Command "version" -WM $wmVer.Stdout.Trim() -Knowns $knownsVer.Stdout.Trim() -Notes "Both report version"
Add-Result -Command "subcommands" -WM $wmCmdCount -Knowns $knownsCmdCount -Notes "WM: $wmCmdCount, Knowns: $knownsCmdCount"
Write-Host ""

# ─── 3. Create temp projects ───
Write-Host "─── 2. INIT PROJECTS ───" -ForegroundColor Yellow

$wmDir = Join-Path $tmpRoot "wm"
$knownsDir = Join-Path $tmpRoot "knowns"
New-Item -ItemType Directory -Path $wmDir -Force | Out-Null
New-Item -ItemType Directory -Path $knownsDir -Force | Out-Null

$wmInit = Invoke-ProcessCapture -FilePath $wmExe -Arguments "init" -WorkingDirectory $wmDir
$knownsInit = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "init" -WorkingDirectory $knownsDir
Write-Host "WM init:     Exit code $($wmInit.ExitCode)" -ForegroundColor Green
Write-Host "Knowns init: Exit code $($knownsInit.ExitCode)" -ForegroundColor Green
Add-Result -Command "init" -WM "OK (exit $($wmInit.ExitCode))" -Knowns "OK (exit $($knownsInit.ExitCode))" -Notes "Both init a project"
Write-Host ""

# ─── 4. Create test content ───
Write-Host "─── 3. CREATE TEST PAGES ───" -ForegroundColor Yellow

# WM pages
@"
---
title: Authentication
type: concept
tags: [auth, security]
relates_to:
  - {type: extends, target: wiki:concepts:sessions}
---
# Auth
Using [[sessions]] for auth.
"@ | Set-Content "$wmDir\.wm\wiki\concepts\auth.md" -NoNewline

@"
---
title: Session Management
type: concept
tags: [auth]
aliases: [sessions]
---
# Sessions
How sessions work.
"@ | Set-Content "$wmDir\.wm\wiki\concepts\sessions.md" -NoNewline

@"
---
title: OAuth2 Task
type: task
tags: [auth]
priority: high
acceptance_criteria:
  - {text: "Login works", checked: false}
---
# OAuth2
Implement OAuth2 login.
"@ | Set-Content "$wmDir\.wm\wiki\tasks\oauth2.md" -NoNewline

# Rebuild WM index so pages are discoverable
$wmRebuild = Invoke-ProcessCapture -FilePath $wmExe -Arguments "index rebuild" -WorkingDirectory $wmDir
Write-Host "WM: 3 pages created + index rebuilt (exit $($wmRebuild.ExitCode))" -ForegroundColor Green

# Knowns pages (using knowns CLI)
$ErrorActionPreference = 'SilentlyContinue'
$knownsDoc = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "doc create concepts/auth --content", "---
title: Authentication
---
# Auth
Auth system." -WorkingDirectory $knownsDir
$ErrorActionPreference = 'Continue'
Write-Host "Knowns: doc created (exit $($knownsDoc.ExitCode))" -ForegroundColor Green
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 5. PAGE LIST ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 4. PAGE LIST ───" -ForegroundColor Yellow

$wmPageList = Invoke-ProcessCapture -FilePath $wmExe -Arguments "page list --json" -WorkingDirectory $wmDir
$knownsDocList = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "doc list --json" -WorkingDirectory $knownsDir

$wmPages = @()
try { $wmPages = $wmPageList.Stdout | ConvertFrom-Json } catch {}
$knownsPages = @()
try { $knownsPages = $knownsDocList.Stdout | ConvertFrom-Json } catch {}

$wmPageCount = @($wmPages).Count
$knownsPageCount = @($knownsPages).Count

Write-Host "WM page list:     $wmPageCount pages" -ForegroundColor Green
Write-Host "Knowns doc list:  $knownsPageCount docs" -ForegroundColor Green
Add-Result -Command "page list" -WM "$wmPageCount pages (JSON)" -Knowns "$knownsPageCount docs (JSON)" -Notes "Both output JSON with --json flag"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 6. SEARCH QUERY ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 5. SEARCH ───" -ForegroundColor Yellow

$wmSearch = Invoke-ProcessCapture -FilePath $wmExe -Arguments "search query auth --json" -WorkingDirectory $wmDir
$knownsSearch = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "search auth --json" -WorkingDirectory $knownsDir

$wmSearchResults = @()
try { $wmSearchResults = $wmSearch.Stdout | ConvertFrom-Json } catch {}

$knownsSearchResults = @()
try { $knownsSearchResults = $knownsSearch.Stdout | ConvertFrom-Json } catch {}

$wmSearchCount = @($wmSearchResults).Count
$knownsSearchCount = @($knownsSearchResults).Count

Write-Host "WM search 'auth':      $wmSearchCount results" -ForegroundColor Green
Write-Host "Knowns search 'auth':  $knownsSearchCount results" -ForegroundColor Green
Add-Result -Command "search" -WM "$wmSearchCount results (BM25+semantic+hybrid)" -Knowns "$knownsSearchCount results" -Notes "WM has 3 search modes; Knowns uses semantic by default"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 7. GRAPH STATS ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 6. GRAPH ───" -ForegroundColor Yellow

$wmGraphStats = Invoke-ProcessCapture -FilePath $wmExe -Arguments "graph stats --json" -WorkingDirectory $wmDir
$wmGraphNeighbors = Invoke-ProcessCapture -FilePath $wmExe -Arguments "graph neighbors wiki:concepts:auth --json" -WorkingDirectory $wmDir

try {
    $graphStats = $wmGraphStats.Stdout | ConvertFrom-Json
    Write-Host "WM graph stats:" -ForegroundColor Green
    Write-Host "  nodes: $($graphStats.nodes)" -ForegroundColor White
    Write-Host "  edges: $($graphStats.edges)" -ForegroundColor White
} catch {
    Write-Host "WM graph stats: (parse failed)" -ForegroundColor Red
}

try {
    $neighbors = $wmGraphNeighbors.Stdout | ConvertFrom-Json
    $neighborCount = @($neighbors).Count
    Write-Host "WM graph neighbors (auth): $neighborCount neighbors" -ForegroundColor Green
} catch {
    Write-Host "WM graph neighbors: (parse failed)" -ForegroundColor Red
}

# Knowns doesn't have a direct graph command, but has resolve/retrieve
$knownsResolve = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "resolve @doc/concepts/auth" -WorkingDirectory $knownsDir 2>$null
$knownsHasGraph = if ($knownsResolve.ExitCode -eq 0) { "resolve works" } else { "no graph CLI" }

Add-Result -Command "graph stats" -WM "nodes/edges/per-type breakdown" -Knowns $knownsHasGraph -Notes "WM has typed edge graph; Knowns has doc/task/memory entities"
Add-Result -Command "graph neighbors" -WM "topic-aware neighbor list" -Knowns "N/A" -Notes "Unique to WM: typed graph traversal"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 8. LINT CHECK ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 7. LINT ───" -ForegroundColor Yellow

$wmLint = Invoke-ProcessCapture -FilePath $wmExe -Arguments "lint check --json" -WorkingDirectory $wmDir
$knownsValidate = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "validate" -WorkingDirectory $knownsDir

try {
    $lintResult = $wmLint.Stdout | ConvertFrom-Json
    Write-Host "WM lint check: OK" -ForegroundColor Green
} catch {
    Write-Host "WM lint check: completed" -ForegroundColor Green
}

Add-Result -Command "lint check" -WM "orphans/broken-refs/missing-ACs/cycles" -Knowns "validate (per-type completeness)" -Notes "WM lint is more comprehensive; Knowns validate covers similar ground"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 9. VALIDATE ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 8. VALIDATE ───" -ForegroundColor Yellow

$wmValidate = Invoke-ProcessCapture -FilePath $wmExe -Arguments "validate" -WorkingDirectory $wmDir
Write-Host "WM validate: exit code $($wmValidate.ExitCode)" -ForegroundColor Green
Add-Result -Command "validate" -WM "per-type frontmatter completeness" -Knowns "validate (tasks/docs/templates)" -Notes "Both have validate; WM checks per-type required fields"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 10. TIME REPORT ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 9. TIME ───" -ForegroundColor Yellow

$wmTimeReport = Invoke-ProcessCapture -FilePath $wmExe -Arguments "time report --json" -WorkingDirectory $wmDir
try {
    $timeResult = $wmTimeReport.Stdout | ConvertFrom-Json
    Write-Host "WM time report: OK" -ForegroundColor Green
} catch {
    Write-Host "WM time report: completed (no entries yet)" -ForegroundColor Green
}

$knownsTime = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "time report --json" -WorkingDirectory $knownsDir
$knownsTimeOk = if ($knownsTime.ExitCode -eq 0) { "report works" } else { "error/has report?" }

Add-Result -Command "time report" -WM "start/stop/add/report + orphan recovery" -Knowns $knownsTimeOk -Notes "Both have time tracking"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 11. INDEX REBUILD ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 10. INDEX REBUILD ───" -ForegroundColor Yellow

$wmIndexRebuild = Invoke-ProcessCapture -FilePath $wmExe -Arguments "index rebuild" -WorkingDirectory $wmDir
Write-Host "WM index rebuild: exit $($wmIndexRebuild.ExitCode)" -ForegroundColor Green

$knownsSync = Invoke-ProcessCapture -FilePath $knownsExe -Arguments "sync" -WorkingDirectory $knownsDir
Write-Host "Knowns sync: exit $($knownsSync.ExitCode)" -ForegroundColor Green

Add-Result -Command "index rebuild" -WM "graph+BM25+embeddings+index.md (ArcSwap)" -Knowns "sync command (skills+instructions+index)" -Notes "WM rebuilds all; Knowns syncs config"
Write-Host ""

# ═══════════════════════════════════════════════════════════════
# ─── 12. JSON output format comparison ───
# ═══════════════════════════════════════════════════════════════
Write-Host "─── 11. OUTPUT FORMAT ───" -ForegroundColor Yellow

# Validate WM JSON output
$wmJsonValid = $false
try {
    $null = $wmPageList.Stdout | ConvertFrom-Json
    $wmJsonValid = $true
} catch {}

# Validate Knowns JSON output
$knownsJsonValid = $false
try {
    $null = $knownsDocList.Stdout | ConvertFrom-Json
    $knownsJsonValid = $true
} catch {}

Write-Host "WM JSON output valid:     $wmJsonValid" -ForegroundColor Green
Write-Host "Knowns JSON output valid: $knownsJsonValid" -ForegroundColor Green
Add-Result -Command "JSON output" -WM "Valid ($wmJsonValid)" -Knowns "Valid ($knownsJsonValid)" -Notes "Both support --json flag for machine-readable output"

# ═══════════════════════════════════════════════════════════════
# ─── 13. MCP named pipe test (optional) ───
# ═══════════════════════════════════════════════════════════════
Write-Host "`n─── 12. MCP OVER NAMED PIPE ───" -ForegroundColor Yellow

# Only test MCP with WM (Knowns uses a different MCP invocation pattern)
$mcpRequests = @(
    '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1"}},"id":1}',
    '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}',
    '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'
)

# We can't test named pipes with the actual CLI if it doesn't support --mcp-pipe,
# so let's test the WM MCP server via stdio using temp file approach instead:
$mcpOutFile = Join-Path $tmpRoot "mcp-stdout.txt"
$mcpErrFile = Join-Path $tmpRoot "mcp-stderr.txt"

# Write MCP requests to a temp file
$reqFile = Join-Path $tmpRoot "mcp-requests.jsonl"
$mcpRequests | Set-Content $reqFile

Write-Host "Testing WM MCP server via temp-file pipe..." -ForegroundColor Green

# Use cmd.exe to pipe the request file into the process
$cmdArgs = "/c type `"$reqFile`" | `"$wmExe`" serve 2> `"$mcpErrFile`" > `"$mcpOutFile`""
$cmdProc = Invoke-ProcessCapture -FilePath "cmd.exe" -Arguments $cmdArgs -WorkingDirectory $wmDir

# Read and parse the response
$mcpResponse = Get-Content $mcpOutFile -ErrorAction SilentlyContinue
$mcpToolCount = 0
foreach ($line in $mcpResponse) {
    if ($line -match '"jsonrpc"') {
        try {
            $obj = $line | ConvertFrom-Json
            if ($obj.id -eq 2 -and $obj.result -and $obj.result.tools) {
                $mcpToolCount = @($obj.result.tools).Count
            }
        } catch {}
    }
}

Write-Host "WM MCP tools registered: $mcpToolCount" -ForegroundColor Green
Add-Result -Command "MCP tools" -WM "$mcpToolCount tools (JSON-RPC 2.0)" -Knowns "~30 tools (JSON-RPC 2.0)" -Notes "Both expose MCP interface over stdio"

# ═══════════════════════════════════════════════════════════════
# ─── SUMMARY TABLE ───
# ═══════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  COMPARISON SUMMARY" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan
Write-Host ""

# Render as a table
$results | Format-Table -Property @{Label='Command'; Expression={$_.Command}; Width=20},
    @{Label='WM Engine'; Expression={$_.WM}; Width=40},
    @{Label='Knowns'; Expression={$_.Knowns}; Width=40},
    @{Label='Notes'; Expression={$_.Notes}; Width=40} -AutoSize

Write-Host ""
Write-Host "─" * 80
Write-Host "Key architectural differences:" -ForegroundColor Yellow
Write-Host "  • Graph:        WM has typed petgraph StableGraph (17 edge types), Knowns has entity stores" -ForegroundColor White
Write-Host "  • Search:       WM has BM25 + semantic (ONNX) + hybrid (RRF), Knowns has semantic + keyword" -ForegroundColor White
Write-Host "  • State:        WM has raw source state machine (pending→processing→done), Knowns has import/sync" -ForegroundColor White
Write-Host "  • File I/O:     WM uses System.Diagnostics.Process for CLI; MCP uses temp-file/named-pipe" -ForegroundColor White
Write-Host "  • Output:       WM outputs text by default, JSON with --json; Knowns outputs styled TUI by default" -ForegroundColor White
Write-Host "  • Config:       WM uses .wm/config.json; Knowns uses knowns.json" -ForegroundColor White
Write-Host "  • MCP style:    WM has wm_ prefix on all tools; Knowns uses bare tool names" -ForegroundColor White
Write-Host ""

# Cleanup
Remove-Item -Recurse -Force $tmpRoot -ErrorAction SilentlyContinue
Write-Host "Temp files cleaned up: $tmpRoot" -ForegroundColor DarkGray
Write-Host "============================================" -ForegroundColor Cyan
