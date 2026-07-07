import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/pages/+server.ts
async function GET() {
	try {
		return json(await callTool("wm_page.list"));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { GET };
