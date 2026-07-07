import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/source/+server.ts
async function POST({ request }) {
	try {
		const { action, ...args } = await request.json();
		return json(await callTool(`wm_source.${action}`, args));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { POST };
