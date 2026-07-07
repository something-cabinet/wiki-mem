import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/tasks/+server.ts
async function GET() {
	try {
		return json(await callTool("wm_task.board"));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
async function POST({ request }) {
	try {
		const { action, ...args } = await request.json();
		return json(await callTool(`wm_task.${action}`, args));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { GET, POST };
