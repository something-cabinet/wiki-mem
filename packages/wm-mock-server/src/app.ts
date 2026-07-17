import { Hono } from "hono";
import { cors } from "hono/cors";
import { loadStubs, getStubs, addStub, removeAllDynamicStubs, removeDynamicStub, loadScenarioStubs } from "./stub-loader.ts";
import type { ParsedStub, StubMapping } from "./stub-loader.ts";

function matchStub(method: string, url: URL, stubs: ParsedStub[]): ParsedStub | undefined {
	const pathname = url.pathname;
	const candidates = stubs.filter((stub) => {
		if (stub.request.method !== method) return false;
		if (stub.request.urlPath) return stub.request.urlPath === pathname;
		if (stub.urlPathRegex) return stub.urlPathRegex.test(pathname);
		return false;
	});
	if (candidates.length === 0) return undefined;
	// Prefer stub with exact matching query parameters
	for (const stub of candidates) {
		if (!stub.request.queryParameters) continue;
		let allMatch = true;
		for (const [key, expected] of Object.entries(stub.request.queryParameters)) {
			const actual = url.searchParams.getAll(key);
			const expectedArr = Array.isArray(expected) ? expected : [expected];
			if (actual.length !== expectedArr.length) { allMatch = false; break; }
			if (!actual.every(v => expectedArr.includes(v))) { allMatch = false; break; }
		}
		if (allMatch) return stub;
	}
	return candidates.find(s => !s.request.queryParameters);
}

export function createApp(mappingsDir: string): Hono {
	loadStubs(mappingsDir);
	const app = new Hono();
	app.use("*", cors());

	// ── Admin API ──
	app.get("/__admin/health", (c) => c.text("OK"));

	app.get("/__admin/mappings", (c) => {
		const loaded = getStubs();
		const list = loaded.map((s) => ({
			id: s.id,
			request: { method: s.request.method, urlPath: s.request.urlPath, urlPathPattern: s.request.urlPathPattern },
		}));
		return c.json({ mappings: list, meta: { total: list.length } });
	});

	app.post("/__admin/mappings/reset", (c) => {
		try {
			removeAllDynamicStubs();
			loadStubs(mappingsDir);
			return c.json({ status: "ok", message: `Reloaded ${getStubs().length} stubs` });
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			return c.json({ status: "error", message }, 500 as const);
		}
	});

	app.post("/__admin/shutdown", (c) => {
		console.log("Shutting down mock server...");
		setTimeout(() => process.exit(0), 100);
		return c.text("OK");
	});

	app.post("/__admin/mappings", async (c) => {
		try {
			const body = (await c.req.json()) as Record<string, unknown>;
			const request = body.request as Record<string, unknown> | undefined;
			const response = body.response as Record<string, unknown> | undefined;
			if (!request || typeof request !== "object") {
				return c.json({ status: "error", message: "Missing 'request' object" }, 400 as const);
			}
			if (typeof request.method !== "string" || request.method !== request.method.toUpperCase()) {
				return c.json({ status: "error", message: "'request.method' must be an uppercase HTTP method" }, 400 as const);
			}
			if (!request.urlPath && !request.urlPathPattern) {
				return c.json({ status: "error", message: "'request.urlPath' or 'request.urlPathPattern' is required" }, 400 as const);
			}
			if (!response || typeof response !== "object") {
				return c.json({ status: "error", message: "Missing 'response' object" }, 400 as const);
			}
			if (typeof response.status !== "number") {
				return c.json({ status: "error", message: "'response.status' must be a number" }, 400 as const);
			}
			const stub: StubMapping = {
				id: (body.id as string) || `dynamic-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
				request: {
					method: request.method as string,
					urlPath: request.urlPath as string | undefined,
					urlPathPattern: request.urlPathPattern as string | undefined,
					queryParameters: request.queryParameters as Record<string, string | string[]> | undefined,
				},
				response: {
					status: response.status as number,
					headers: (response.headers as Record<string, string>) ?? {},
					jsonBody: response.jsonBody,
				},
			};
			const parsed = addStub(stub);
			return c.json({ status: "ok", message: "Stub registered", id: parsed.id });
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			return c.json({ status: "error", message }, 400 as const);
		}
	});

	app.delete("/__admin/mappings", (c) => {
		removeAllDynamicStubs();
		return c.json({ status: "ok", message: "Dynamic stubs cleared" });
	});

	app.delete("/__admin/mappings/:id", (c) => {
		const id = c.req.param("id");
		const removed = removeDynamicStub(id);
		if (removed) {
			return c.json({ status: "ok", message: `Stub ${id} removed` });
		}
		return c.json({ status: "error", message: `Stub not found: ${id}` }, 404 as const);
	});

	app.post("/__admin/scenarios/reset", (c) => {
		removeAllDynamicStubs();
		return c.json({ status: "ok", message: "All scenarios reset" });
	});

	app.post("/__admin/scenarios/:name/activate", (c) => {
		try {
			const name = c.req.param("name");
			const loaded = loadScenarioStubs(mappingsDir, name);
			return c.json({ status: "ok", message: `Scenario "${name}" activated`, count: loaded.length });
		} catch (err) {
			const message = err instanceof Error ? err.message : String(err);
			return c.json({ status: "error", message }, 400 as const);
		}
	});

	// ── Catch-all: match stubs ──
	app.all("*", async (c) => {
		try {
			const method = c.req.method;
			const url = new URL(c.req.url);
			const matched = matchStub(method, url, getStubs());
			if (!matched) {
				return c.json({ error: "No matching stub found", path: url.pathname, method }, 404 as const);
			}
			const headers: Record<string, string> = { ...matched.response.headers };
			return new Response(JSON.stringify(matched.response.jsonBody), {
				status: matched.response.status as 200,
				headers,
			});
		} catch (err) {
			console.error("Error processing request:", err);
			return c.json({ error: "Internal server error" }, 500 as const);
		}
	});

	return app;
}
