import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/rebuild/+server.ts
async function POST() {
	try {
		return json(await callTool("wm_index.rebuild"));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { POST };
