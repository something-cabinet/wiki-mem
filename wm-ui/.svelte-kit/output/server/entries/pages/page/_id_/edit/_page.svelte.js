import { l as stringify, m as attr, u as unsubscribe_stores } from "../../../../../chunks/index-server.js";
import "../../../../../chunks/stores.js";
import "../../../../../chunks/toasts.js";
import "../../../../../chunks/navigation.js";
//#region src/routes/page/[id]/edit/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		$$renderer.push(`<div class="page svelte-ykrk1d"><a${attr("href", `/page/${stringify(encodeURIComponent(""))}`)} class="back svelte-ykrk1d">← Back to page</a> <h1 class="svelte-ykrk1d">Edit Page</h1> `);
		$$renderer.push("<!--[0-->");
		$$renderer.push(`<div class="skeleton" style="height: 2rem; width: 12rem; margin-bottom: 1rem;"></div> <div class="skeleton" style="height: 2rem; width: 100%; margin-bottom: 1rem;"></div> <div class="skeleton" style="height: 2rem; width: 100%; margin-bottom: 1rem;"></div> <div class="skeleton" style="height: 300px; width: 100%;"></div>`);
		$$renderer.push(`<!--]--></div>`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}
//#endregion
export { _page as default };
