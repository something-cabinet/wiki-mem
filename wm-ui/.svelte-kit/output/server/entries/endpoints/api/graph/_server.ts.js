import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/graph/+server.ts
async function GET({ url }) {
	const center = url.searchParams.get("center");
	const depth = parseInt(url.searchParams.get("depth") || "1");
	try {
		if (center) return json(await callTool("wm_graph.subgraph", {
			center,
			depth
		}));
		return json(await callTool("wm_graph.stats"));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { GET };
