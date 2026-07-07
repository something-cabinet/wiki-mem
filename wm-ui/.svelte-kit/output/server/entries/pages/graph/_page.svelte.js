import { m as attr } from "../../../chunks/index-server.js";
//#region src/routes/graph/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		$$renderer.push(`<div class="page svelte-315y67"><h1 class="svelte-315y67">Graph View</h1> <div class="controls svelte-315y67"><input type="text" placeholder="Center page ID (optional)"${attr("value", "")} class="svelte-315y67"/> <span class="hint svelte-315y67">Leave empty to show all pages at depth 1</span></div> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<div class="skeleton" style="height: 500px; border-radius: var(--radius-md);"></div>`);
		$$renderer.push(`<!--]--></div>`);
	});
}
//#endregion
export { _page as default };
