import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { StubMapping, ParsedStub } from "./core/types";

// ── Existing exports kept for backward compatibility ──
export type { StubRequest, StubResponse, StubMapping, ParsedStub } from "./core/types";
export { MockRegistry } from "./core/registry";
export { matchStub } from "./core/matcher";

let stubs: ParsedStub[] = [];
let dynamicStubs: ParsedStub[] = [];

export function getStubs() { return [...dynamicStubs, ...stubs]; }

export function addStub(mapping: StubMapping) {
  const parsed: ParsedStub = { ...mapping };
  if (mapping.request.urlPathPattern) parsed.urlPathRegex = new RegExp(mapping.request.urlPathPattern);
  dynamicStubs.push(parsed);
  return parsed;
}

export function removeAllDynamicStubs() { dynamicStubs = []; }

export function removeDynamicStub(id: string) {
  const index = dynamicStubs.findIndex(s => s.id === id);
  if (index === -1) return false;
  dynamicStubs.splice(index, 1);
  return true;
}

export function loadScenarioStubs(mappingsDir: string, scenarioName: string) {
  const scenarioDir = resolve(mappingsDir, scenarioName);
  if (!existsSync(scenarioDir)) throw new Error(`Scenario directory not found: ${scenarioDir}`);
  const files = readdirSync(scenarioDir).filter(f => f.endsWith(".json"));
  const loaded: StubMapping[] = [];
  for (const file of files) {
    const raw = readFileSync(resolve(scenarioDir, file), "utf-8");
    const rawJson = JSON.parse(raw);
    validateStub(rawJson, file);
    const req = rawJson.request as any;
    const res = rawJson.response as any;
    const stub: StubMapping = {
      id: `${scenarioName}-${file.replace(".json", "")}`,
      request: { method: req.method, urlPath: req.urlPath, urlPathPattern: req.urlPathPattern, queryParameters: req.queryParameters },
      response: { status: res.status, headers: res.headers ?? {}, jsonBody: res.jsonBody },
    };
    const parsed: ParsedStub = { ...stub };
    if (stub.request.urlPathPattern) parsed.urlPathRegex = new RegExp(stub.request.urlPathPattern);
    dynamicStubs.push(parsed);
    loaded.push(stub);
  }
  return loaded;
}

export function loadStubs(mappingsDir: string) {
  const absoluteDir = resolve(mappingsDir);
  const files = readdirSync(absoluteDir).filter(f => f.endsWith(".json"));
  const loaded: StubMapping[] = [];
  for (const file of files) {
    const raw = readFileSync(resolve(absoluteDir, file), "utf-8");
    const parsed = JSON.parse(raw);
    validateStub(parsed, file);
    const req = parsed.request as any;
    const res = parsed.response as any;
    const stub: StubMapping = {
      id: file.replace(".json", ""),
      request: { method: req.method, urlPath: req.urlPath, urlPathPattern: req.urlPathPattern, queryParameters: req.queryParameters },
      response: { status: res.status, headers: res.headers ?? {}, jsonBody: res.jsonBody },
    };
    loaded.push(stub);
  }
  stubs = loaded.map(s => {
    const p: ParsedStub = { ...s };
    if (s.request.urlPathPattern) p.urlPathRegex = new RegExp(s.request.urlPathPattern);
    return p;
  });
  return loaded;
}

function validateStub(stub: unknown, filename: string) {
  if (!stub || typeof stub !== "object") throw new Error(`Stub ${filename} is not a valid JSON object`);
  const s = stub as any;
  if (!s.request || typeof s.request !== "object") throw new Error(`Stub ${filename} missing "request" object`);
  if (typeof s.request.method !== "string") throw new Error(`Stub ${filename} missing "request.method" string`);
  if (!s.request.urlPath && !s.request.urlPathPattern) throw new Error(`Stub ${filename} must have "request.urlPath" or "request.urlPathPattern"`);
  if (!s.response || typeof s.response !== "object") throw new Error(`Stub ${filename} missing "response" object`);
  if (typeof s.response.status !== "number") throw new Error(`Stub ${filename} missing "response.status" number`);
}
