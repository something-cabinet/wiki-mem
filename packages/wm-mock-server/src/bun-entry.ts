#!/usr/bin/env bun

import { createApp } from "./app.ts";

function parseArgs(args: string[]): { mappings: string; port: number } {
	let mappings = "";
	let port = 8081;
	for (let i = 0; i < args.length; i++) {
		if (args[i] === "--mappings" && i + 1 < args.length) {
			mappings = args[++i]!;
		} else if (args[i] === "--port" && i + 1 < args.length) {
			const parsed = parseInt(args[++i]!, 10);
			if (!isNaN(parsed) && parsed > 0) port = parsed;
		}
	}
	if (!mappings) {
		console.error("Error: --mappings <path> is required");
		process.exit(1);
	}
	return { mappings, port };
}

const args = parseArgs(process.argv.slice(2));
console.log(`Mock server starting on port ${args.port} with mappings: ${args.mappings}`);
const app = createApp(args.mappings);

Bun.serve({
	port: args.port,
	fetch: app.fetch,
	error(error) {
		console.error("Mock server error:", error);
		return new Response("Internal Server Error", { status: 500 });
	},
});

console.log(`Mock server listening on http://localhost:${args.port}`);
