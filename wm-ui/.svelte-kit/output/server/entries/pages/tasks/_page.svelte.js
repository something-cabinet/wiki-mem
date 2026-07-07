import { o as ensure_array_like } from "../../../chunks/index-server.js";
import "../../../chunks/toasts.js";
//#region src/routes/tasks/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		$$renderer.push(`<div class="page svelte-1pluywh"><h1 class="svelte-1pluywh">Task Board</h1> `);
		{
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="board svelte-1pluywh"><!--[-->`);
			const each_array = ensure_array_like([
				"todo",
				"in_progress",
				"done",
				"blocked"
			]);
			for (let $$index_1 = 0, $$length = each_array.length; $$index_1 < $$length; $$index_1++) {
				each_array[$$index_1];
				$$renderer.push(`<div class="column svelte-1pluywh"><div class="column-header svelte-1pluywh"><div class="skeleton" style="height: 1rem; width: 4rem;"></div> <div class="skeleton" style="height: 1rem; width: 1.5rem;"></div></div> <div class="cards svelte-1pluywh"><!--[-->`);
				const each_array_1 = ensure_array_like([1, 2]);
				for (let $$index = 0, $$length = each_array_1.length; $$index < $$length; $$index++) {
					each_array_1[$$index];
					$$renderer.push(`<div class="card svelte-1pluywh" style="border-left-color: transparent;"><div class="skeleton" style="height: 1rem; margin-bottom: 0.5rem;"></div> <div class="skeleton" style="height: 0.75rem; width: 3rem;"></div></div>`);
				}
				$$renderer.push(`<!--]--></div></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		}
		$$renderer.push(`<!--]--></div>`);
	});
}
//#endregion
export { _page as default };
