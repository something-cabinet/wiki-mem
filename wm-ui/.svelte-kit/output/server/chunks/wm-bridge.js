import { spawn } from "node:child_process";
import { createInterface } from "node:readline";
//#region src/lib/server/wm-bridge.ts
var wmProcess = null;
var requestId = 1;
var pending = /* @__PURE__ */ new Map();
async function startWm(projectRoot) {
	if (wmProcess) return;
	const args = [
		"run",
		"--",
		"serve"
	];
	if (projectRoot) args.push("--project", projectRoot);
	wmProcess = spawn(process.env.WM_BIN || "../target/debug/wm-cli.exe", ["serve", ...projectRoot ? ["--project", projectRoot] : []], {
		cwd: projectRoot || process.cwd(),
		stdio: [
			"pipe",
			"pipe",
			"pipe"
		],
		windowsHide: true
	});
	createInterface({ input: wmProcess.stdout }).on("line", (line) => {
		try {
			const msg = JSON.parse(line);
			if (msg.id && pending.has(msg.id)) {
				const p = pending.get(msg.id);
				pending.delete(msg.id);
				if (msg.error) p.reject(new Error(msg.error.message || JSON.stringify(msg.error)));
				else p.resolve(msg.result);
			}
		} catch {}
	});
	wmProcess.stderr?.on("data", (data) => {
		if (process.env.DEBUG_WM) console.error(`[wm] ${data.toString().trim()}`);
	});
	wmProcess.on("exit", (code) => {
		console.error(`wm serve exited with code ${code}`);
		wmProcess = null;
	});
	await sendRequest("initialize", {});
}
async function stopWm() {
	if (wmProcess) {
		wmProcess.kill();
		wmProcess = null;
	}
}
async function callTool(name, args = {}) {
	if (!wmProcess) await startWm();
	return sendRequest("tools/call", {
		name,
		arguments: args
	});
}
async function sendRequest(method, params) {
	return new Promise((resolve, reject) => {
		const id = requestId++;
		const request = JSON.stringify({
			jsonrpc: "2.0",
			method,
			params,
			id
		});
		pending.set(id, {
			resolve,
			reject
		});
		if (!wmProcess || !wmProcess.stdin) {
			reject(/* @__PURE__ */ new Error("wm process not running"));
			return;
		}
		wmProcess.stdin.write(request + "\n");
		setTimeout(() => {
			if (pending.has(id)) {
				pending.delete(id);
				reject(/* @__PURE__ */ new Error("Request timed out"));
			}
		}, 3e4);
	});
}
//#endregion
export { startWm as n, stopWm as r, callTool as t };
