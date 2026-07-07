import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/search/+server.ts
async function GET({ url }) {
	const q = url.searchParams.get("q") || "";
	const mode = url.searchParams.get("mode") || "hybrid";
	const limit = parseInt(url.searchParams.get("limit") || "20");
	try {
		return json(await callTool("wm_search.query", {
			q,
			mode,
			limit
		}));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { GET };
