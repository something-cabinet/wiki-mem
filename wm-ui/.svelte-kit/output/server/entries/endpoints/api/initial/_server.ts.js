import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/initial/+server.ts
async function GET() {
	try {
		return json(await callTool("wm_initial"));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { GET };
