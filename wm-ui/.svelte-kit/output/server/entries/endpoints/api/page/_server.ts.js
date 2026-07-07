import { t as callTool } from "../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/page/+server.ts
async function POST({ request }) {
	try {
		const { path, title, content, type } = await request.json();
		return json(await callTool("wm_page.create", {
			path,
			title,
			content,
			type
		}));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { POST };
