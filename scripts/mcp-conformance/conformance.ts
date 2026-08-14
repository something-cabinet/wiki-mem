#!/usr/bin/env node
/**
 * MCP SDK conformance smoke test — Oracle finding m-7.
 *
 * Drives the official TypeScript MCP SDK (`@modelcontextprotocol/sdk`,
 * pinned in package.json) against a live `wm-server` daemon's `/mcp`
 * endpoint over Streamable-HTTP: initialize -> tools/list -> tools/call.
 *
 * Why this exists: the HTTP MCP transport was only ever exercised with raw
 * HTTP (apps/wm-core/tests/mcp_http.rs). This proves a real, unmodified MCP
 * client SDK can complete the handshake and round-trip tool calls — the
 * interoperability claim of the spec's Streamable-HTTP shape.
 *
 * Scope (m-7 note): covers initialize / tools/list / tools/call only.
 * Session-id + GET (server->client) transport is NOT covered — the daemon
 * implements the stateless subset of Streamable-HTTP, so the SDK never
 * opens a server->client stream here. Re-evaluate against rmcp (the Rust
 * MCP SDK) at the next protocol bump; rmcp is the natural next candidate
 * once the daemon grows a real session layer.
 *
 * Known negotiation gap (reported, not fixed here — do NOT edit the Rust
 * server in this task): the SDK (1.30.x) defaults to protocolVersion
 * 2025-11-25, which the daemon does not recognize; the daemon's
 * initialize handler hardcodes a fallback to 2024-11-05 instead of
 * negotiating to the highest mutually-supported version (2025-06-18).
 * 2024-11-05 is still inside the SDK's supported set, so connect()
 * succeeds — but the negotiated version is lower than necessary and would
 * break if a future SDK dropped 2024-11-05. The script's explicit probe
 * requests 2025-06-18 and asserts the daemon echoes it verbatim.
 *
 * Hermetic: creates its own fixture project under a temp dir, spawns the
 * daemon with `--port <free>`, waits for /api/health, reads the web token
 * the daemon writes to .wm/state/web-token, then drives the SDK. No
 * network, no real API keys. The daemon and fixture are torn down in all
 * paths (try/finally).
 *
 * Usage:
 *   node conformance.ts                                # spawns target/debug/wm-server
 *   node conformance.ts --server /abs/path/wm-server   # custom binary
 *   node conformance.ts --base-url http://127.0.0.1:PORT --token <tok>  # external daemon
 */

import { spawn } from "node:child_process";
import type { ChildProcess } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { once } from "node:events";

import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StreamableHTTPClientTransport } from "@modelcontextprotocol/sdk/client/streamableHttp.js";

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));

/** Server's accepted protocol versions (mirrors apps/wm-server/src/routes/mcp.rs). */
const SERVER_PROTOCOLS = ["2024-11-05", "2025-03-26", "2025-06-18"] as const;
/** Version this probe asserts the daemon echoes verbatim. */
const PROBE_PROTOCOL = "2025-06-18";

const MCP_PATH = "/mcp";
const TOKEN_HEADER = "x-wm-token";

function fail(msg: string): never {
  throw new Error(`ASSERTION FAILED: ${msg}`);
}

function assert(cond: unknown, msg: string): asserts cond {
  if (!cond) fail(msg);
}

function parseArgs(argv: string[]) {
  const args: Record<string, string> = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith("--") && argv[i + 1] && !argv[i + 1].startsWith("--")) {
      args[a.slice(2)] = argv[i + 1];
      i++;
    }
  }
  return args;
}

async function freePort(): Promise<number> {
  const server = createServer();
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => resolve());
  });
  const port = (server.address() as { port: number }).port;
  await new Promise<void>((resolve) => server.close(() => resolve()));
  return port;
}

/** Minimal but real fixture project (mirrors apps/wm-core/tests/helpers/setup.rs). */
function makeFixture(dir: string) {
  const wm = join(dir, ".wm");
  for (const sub of [
    "wiki/tasks",
    "wiki/specs",
    "wiki/concepts",
    "wiki/patterns",
    "wiki/decisions",
    "wiki/howto",
    "wiki/reference",
    "wiki/core",
    "sources",
    "state",
    "memory",
  ]) {
    mkdirSync(join(wm, sub), { recursive: true });
  }
  mkdirSync(join(dir, ".agents", "skills"), { recursive: true });

  const config = {
    project_name: "",
    schema_version: 1,
    embedding: { model_name: "bge-small-en-v1.5", dimensions: 384, batch_size: 32 },
    permissions: { preset: "read-write" },
    custom_edge_types: [],
    source_dirs: ["docs/", "specs/"],
    source_extensions: ["md", "yaml", "txt"],
    search: { default_mode: "hybrid", default_limit: 20, rrf_k: 60 },
  };
  writeFileSync(join(wm, "config.json"), JSON.stringify(config, null, 2));

  // Seeded before the daemon boots so the startup rebuild_wiki indexes it —
  // search is then deterministic with no watcher race.
  writeFileSync(
    join(wm, "wiki", "concepts", "http-mcp.md"),
    "---\ntitle: HTTP MCP\ntype: concept\ntags: [mcp]\n---\n\nZirconium-http-mcp unique searchable phrase.\n",
  );
  writeFileSync(join(wm, "AGENTS.md"), "# AGENTS.md — Wiki Memory Engine Agent Handbook\n");
}

async function waitForHealth(
  baseUrl: string,
  child: ChildProcess,
  getSpawnError: () => Error | null,
  timeoutMs = 30_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const spawnErr = getSpawnError();
    if (spawnErr) {
      fail(`failed to spawn wm-server: ${spawnErr.message}`);
    }
    try {
      const resp = await fetch(`${baseUrl}/api/health`);
      if (resp.ok) return;
    } catch {
      // connection refused until the daemon binds — keep polling
    }
    if (child.exitCode !== null) {
      fail(`wm-server exited early (code ${child.exitCode}) — see stderr above`);
    }
    if (Date.now() > deadline) {
      fail(`wm-server did not become healthy within ${timeoutMs}ms`);
    }
    await new Promise((r) => setTimeout(r, 100));
  }
}

/**
 * Explicit initialize probe through the SDK's own transport: requests
 * PROBE_PROTOCOL and asserts the daemon echoes it verbatim (the protocol
 * handshake contract the raw-HTTP suite checks with 2024-11-05).
 */
async function probeInitialize(baseUrl: string, token: string) {
  const transport = new StreamableHTTPClientTransport(new URL(`${baseUrl}${MCP_PATH}`), {
    requestInit: { headers: { [TOKEN_HEADER]: token } },
  });

  const response = await new Promise<Record<string, unknown>>((resolve, reject) => {
    transport.onmessage = (msg) => resolve(msg as Record<string, unknown>);
    transport.onerror = (err) => reject(err);
    transport
      .start()
      .then(() =>
        transport.send({
          jsonrpc: "2.0",
          id: 1,
          method: "initialize",
          params: {
            protocolVersion: PROBE_PROTOCOL,
            capabilities: {},
            clientInfo: { name: "wm-mcp-conformance", version: "1.0.0" },
          },
        }),
      )
      .catch(reject);
  });
  await transport.close();

  const result = response.result as Record<string, unknown> | undefined;
  assert(result, "initialize probe returned no result");
  assert(
    result.protocolVersion === PROBE_PROTOCOL,
    `initialize protocolVersion must echo ${PROBE_PROTOCOL}, got ${JSON.stringify(result.protocolVersion)}`,
  );
  const serverInfo = result.serverInfo as Record<string, unknown> | undefined;
  assert(serverInfo && serverInfo.name === "wm-engine", `unexpected serverInfo: ${JSON.stringify(serverInfo)}`);
  return { protocolVersion: result.protocolVersion as string, serverInfo: serverInfo as Record<string, string> };
}

/** Real SDK client flow: connect() (auto-initialize), tools/list, tools/call. */
async function clientFlow(baseUrl: string, token: string) {
  const transport = new StreamableHTTPClientTransport(new URL(`${baseUrl}${MCP_PATH}`), {
    requestInit: { headers: { [TOKEN_HEADER]: token } },
  });
  const client = new Client({ name: "wm-mcp-conformance", version: "1.0.0" });
  await client.connect(transport); // performs initialize; negotiates to the daemon's answer

  const version = client.getServerVersion();
  assert(version?.name === "wm-engine", `client serverInfo.name=${JSON.stringify(version?.name)}`);
  assert(version?.version, "client serverInfo.version missing");

  const caps = client.getServerCapabilities();
  assert(caps?.tools, "server must advertise the tools capability");

  const { tools } = await client.listTools();
  const names = tools.map((t) => t.name);
  assert(
    names.includes("wm_search.query"),
    `wm_search.query missing from tools/list (${names.length} tools: ${names.join(", ")})`,
  );
  assert(names.includes("wm_initial"), "wm_initial missing from tools/list");

  const call = await client.callTool({
    name: "wm_search.query",
    arguments: { q: "zirconium-http-mcp", type: "all", limit: 10 },
  });
  assert(call.isError === false, `tools/call returned isError=true: ${JSON.stringify(call)}`);
  const text = (call.content as { type: string; text?: string }[] | undefined)
    ?.find((c) => c.type === "text")?.text;
  assert(typeof text === "string" && text.length > 0, "tools/call returned no text content");
  const payload = JSON.parse(text) as { results?: unknown[] };
  assert(Array.isArray(payload.results), `result payload missing results array: ${text.slice(0, 200)}`);
  assert(payload.results.length >= 1, `expected >=1 result for seeded phrase, got ${payload.results.length}`);

  await client.close();
  return { serverInfo: version, toolCount: names.length, resultCount: payload.results.length };
}

async function main() {
  const args = parseArgs(process.argv.slice(2));
  let baseUrl = args["base-url"];
  const externalToken = args["token"];
  const serverBin = args["server"] ?? join(SCRIPT_DIR, "..", "..", "target", "debug", "wm-server");

  let fixture: string | undefined;
  let child: ChildProcess | undefined;
  let port = 0;

  try {
    let token = externalToken;
    if (!baseUrl || !token) {
      assert(!baseUrl || !token, "external-daemon mode needs BOTH --base-url and --token");
      port = await freePort();
      fixture = mkdtempSync(join(tmpdir(), "wm-mcp-conformance-"));
      makeFixture(fixture);

      child = spawn(serverBin, ["--port", String(port)], {
        cwd: fixture,
        stdio: ["ignore", "pipe", "pipe"],
      });
      let spawnError: Error | null = null;
      child.on("error", (err) => (spawnError = err));
      let stderrBuf = "";
      child.stderr?.on("data", (d: Buffer) => (stderrBuf += d.toString()));
      child.stdout?.on("data", () => {});

      baseUrl = `http://127.0.0.1:${port}`;
      await waitForHealth(baseUrl, child, () => spawnError);
      token = readFileSync(join(fixture, ".wm", "state", "web-token"), "utf8").trim();
      console.log(`daemon healthy: ${serverBin} --port ${port} (fixture ${fixture})`);
      if (stderrBuf.trim()) console.log(`[daemon] ${stderrBuf.trim().split("\n").slice(0, 3).join("\n[daemon] ")}`);
    } else {
      console.log(`using external daemon at ${baseUrl}`);
    }

    // 1. Explicit initialize probe: assert the daemon echoes a supported version.
    const probe = await probeInitialize(baseUrl, token);
    console.log(
      `initialize probe: requested ${PROBE_PROTOCOL} -> echoed ${probe.protocolVersion} (serverInfo ${probe.serverInfo.name} ${probe.serverInfo.version})`,
    );

    // 2. Real SDK client flow.
    const flow = await clientFlow(baseUrl, token);
    console.log(
      `client connect OK: ${flow.serverInfo.name} ${flow.serverInfo.version}; ${flow.toolCount} tools listed; wm_search.query + wm_initial present`,
    );
    console.log(`tools/call wm_search.query: isError=false, ${flow.resultCount} result(s) for seeded phrase`);
    console.log("PASS: MCP SDK conformance (initialize / tools/list / tools/call)");
  } finally {
    if (child && child.exitCode === null) {
      child.kill("SIGTERM");
      await Promise.race([once(child, "exit"), new Promise((r) => setTimeout(r, 3_000))]);
      if (child.exitCode === null) child.kill("SIGKILL");
    }
    if (fixture) rmSync(fixture, { recursive: true, force: true });
  }
}

main().catch((err) => {
  console.error(err instanceof Error ? err.message : err);
  process.exit(1);
});
