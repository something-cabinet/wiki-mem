import { t as callTool } from "../../../../../chunks/wm-bridge.js";
import { json } from "@sveltejs/kit";
//#region src/routes/api/page/[id]/+server.ts
async function GET({ params }) {
	const id = params.id;
	if (!id) return json({ error: "Missing id" }, { status: 400 });
	try {
		return json(await callTool("wm_page.get", { id }));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
async function PUT({ params, request }) {
	const id = params.id;
	if (!id) return json({ error: "Missing id" }, { status: 400 });
	try {
		return json(await callTool("wm_page.update", {
			id,
			...await request.json()
		}));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
async function DELETE({ params }) {
	const id = params.id;
	if (!id) return json({ error: "Missing id" }, { status: 400 });
	try {
		return json(await callTool("wm_page.delete", { id }));
	} catch (e) {
		return json({ error: e.message }, { status: 500 });
	}
}
//#endregion
export { DELETE, GET, PUT };
