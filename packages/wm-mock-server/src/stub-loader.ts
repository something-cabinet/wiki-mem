import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

export interface StubRequest {
	method: string;
	urlPath?: string;
	urlPathPattern?: string;
	queryParameters?: Record<string, string | string[]>;
}

export interface StubResponse {
	status: number;
	headers: Record<string, string>;
	jsonBody: unknown;
}

export interface StubMapping {
	id: string;
	request: StubRequest;
	response: StubResponse;
}

export interface ParsedStub {
	id: string;
	request: StubRequest;
	response: StubResponse;
	urlPathRegex?: RegExp;
}

let stubs: ParsedStub[] = [];
let dynamicStubs: ParsedStub[] = [];

export function getStubs(): ParsedStub[] {
	return [...dynamicStubs, ...stubs];
}

export function addStub(mapping: StubMapping): ParsedStub {
	const parsed: ParsedStub = { ...mapping };
	if (mapping.request.urlPathPattern) {
		parsed.urlPathRegex = new RegExp(mapping.request.urlPathPattern);
	}
	dynamicStubs.push(parsed);
	return parsed;
}

export function removeAllDynamicStubs(): void {
	dynamicStubs = [];
}

export function removeDynamicStub(id: string): boolean {
	const index = dynamicStubs.findIndex((s) => s.id === id);
	if (index === -1) return false;
	dynamicStubs.splice(index, 1);
	return true;
}

export function loadScenarioStubs(mappingsDir: string, scenarioName: string): StubMapping[] {
	const scenarioDir = resolve(mappingsDir, scenarioName);
	if (!existsSync(scenarioDir)) {
		throw new Error(`Scenario directory not found: ${scenarioDir}`);
	}
	const files = readdirSync(scenarioDir).filter((f) => f.endsWith(".json"));
	if (files.length === 0) {
		throw new Error(`No JSON stub files found in ${scenarioDir}`);
	}
	const loaded: StubMapping[] = [];
	for (const file of files) {
		const filePath = resolve(scenarioDir, file);
		const raw = readFileSync(filePath, "utf-8");
		const rawJson = JSON.parse(raw);
		validateStub(rawJson, file);
		const req = rawJson.request as Record<string, unknown>;
		const res = rawJson.response as Record<string, unknown>;
		const stub: StubMapping = {
			id: `${scenarioName}-${file.replace(".json", "")}`,
			request: {
				method: req.method as string,
				urlPath: req.urlPath as string | undefined,
				urlPathPattern: req.urlPathPattern as string | undefined,
				queryParameters: req.queryParameters as Record<string, string | string[]> | undefined,
			},
			response: {
				status: res.status as number,
				headers: (res.headers as Record<string, string>) ?? {},
				jsonBody: res.jsonBody,
			},
		};
		const parsed: ParsedStub = { ...stub };
		if (stub.request.urlPathPattern) {
			parsed.urlPathRegex = new RegExp(stub.request.urlPathPattern);
		}
		dynamicStubs.push(parsed);
		loaded.push(stub);
	}
	console.log(`Loaded ${loaded.length} stubs from scenario "${scenarioName}"`);
	return loaded;
}

export function loadStubs(mappingsDir: string): StubMapping[] {
	const absoluteDir = resolve(mappingsDir);
	const files = readdirSync(absoluteDir).filter((f) => f.endsWith(".json"));
	if (files.length === 0) {
		throw new Error(`No JSON stub files found in ${absoluteDir}`);
	}
	const loaded: StubMapping[] = [];
	for (const file of files) {
		const filePath = resolve(absoluteDir, file);
		try {
			const raw = readFileSync(filePath, "utf-8");
			const parsed = JSON.parse(raw);
			validateStub(parsed, file);
			const req = parsed.request as Record<string, unknown>;
			const res = parsed.response as Record<string, unknown>;
			const stub: StubMapping = {
				id: file.replace(".json", ""),
				request: {
					method: req.method as string,
					urlPath: req.urlPath as string | undefined,
					urlPathPattern: req.urlPathPattern as string | undefined,
					queryParameters: req.queryParameters as Record<string, string | string[]> | undefined,
				},
				response: {
					status: res.status as number,
					headers: (res.headers as Record<string, string>) ?? {},
					jsonBody: res.jsonBody,
				},
			};
			loaded.push(stub);
		} catch (err: unknown) {
			const message = err instanceof Error ? err.message : String(err);
			throw new Error(`Failed to load stub ${file}: ${message}`);
		}
	}
	stubs = loaded.map((s) => {
		const parsed: ParsedStub = { ...s };
		if (s.request.urlPathPattern) {
			try {
				parsed.urlPathRegex = new RegExp(s.request.urlPathPattern);
			} catch {
				throw new Error(`Invalid regex in urlPathPattern for stub "${s.id}": ${s.request.urlPathPattern}`);
			}
		}
		return parsed;
	});
	console.log(`Loaded ${stubs.length} stubs from ${absoluteDir}`);
	return loaded;
}

function validateStub(stub: unknown, filename: string): asserts stub is { request: Record<string, unknown>; response: Record<string, unknown> } {
	if (!stub || typeof stub !== "object") {
		throw new Error(`Stub ${filename} is not a valid JSON object`);
	}
	const s = stub as Record<string, unknown>;
	if (!s.request || typeof s.request !== "object") {
		throw new Error(`Stub ${filename} missing "request" object`);
	}
	const req = s.request as Record<string, unknown>;
	if (typeof req.method !== "string") {
		throw new Error(`Stub ${filename} missing "request.method" string`);
	}
	if (!req.urlPath && !req.urlPathPattern) {
		throw new Error(`Stub ${filename} must have either "request.urlPath" or "request.urlPathPattern"`);
	}
	if (!s.response || typeof s.response !== "object") {
		throw new Error(`Stub ${filename} missing "response" object`);
	}
	const res = s.response as Record<string, unknown>;
	if (typeof res.status !== "number") {
		throw new Error(`Stub ${filename} missing "response.status" number`);
	}
}
