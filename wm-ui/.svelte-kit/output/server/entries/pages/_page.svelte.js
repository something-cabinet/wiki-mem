import { a as derived, o as ensure_array_like } from "../../chunks/index-server.js";
import "../../chunks/toasts.js";
//#region src/routes/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let pages = [];
		let pageNum = 1;
		const pageSize = 20;
		derived(() => Math.max(1, Math.ceil(pages.length / pageSize)));
		derived(() => pages.slice((pageNum - 1) * pageSize, pageNum * pageSize));
		{
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="stat-grid"><!--[-->`);
			const each_array = ensure_array_like([
				1,
				2,
				3,
				4
			]);
			for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
				each_array[$$index];
				$$renderer.push(`<div class="card stat"><div class="skeleton" style="height: 2rem; width: 3rem; margin: 0 auto;"></div> <div class="skeleton" style="height: 0.875rem; width: 5rem; margin: 0.5rem auto 0;"></div></div>`);
			}
			$$renderer.push(`<!--]--></div> <div class="toolbar svelte-1uha8ag"><div class="search-bar svelte-1uha8ag"><div class="skeleton" style="height: 2rem; width: 100%;"></div> <div class="skeleton" style="height: 2rem; width: 6rem;"></div> <div class="skeleton" style="height: 2rem; width: 4rem;"></div></div> <div class="actions svelte-1uha8ag"><div class="skeleton" style="height: 2rem; width: 6rem;"></div> <div class="skeleton" style="height: 2rem; width: 7rem;"></div></div></div> <div class="tabs skeleton svelte-1uha8ag" style="height: 2rem; margin-bottom: 1rem;"></div> <div class="card" style="padding: 1rem;"><!--[-->`);
			const each_array_1 = ensure_array_like([
				1,
				2,
				3,
				4,
				5
			]);
			for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
				each_array_1[$$index_1];
				$$renderer.push(`<div class="skeleton" style="height: 1.25rem; margin-bottom: 0.5rem;"></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		}
		$$renderer.push(`<!--]--> `);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]-->`);
	});
}
//#endregion
export { _page as default };
