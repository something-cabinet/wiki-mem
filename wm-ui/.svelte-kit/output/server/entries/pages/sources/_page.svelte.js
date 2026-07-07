import { g as escape_html, i as attr_class, l as stringify, m as attr, o as ensure_array_like } from "../../../chunks/index-server.js";
import { t as addToast } from "../../../chunks/toasts.js";
import { t as ConfirmDialog } from "../../../chunks/ConfirmDialog.js";
//#region src/routes/sources/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let sources = [];
		let loading = true;
		let processingId = null;
		let deletingId = null;
		let deleteTarget = null;
		async function loadSources() {
			try {
				sources = (await (await fetch("/api/sources")).json()).sources || [];
			} catch {
				addToast("error", "Failed to load sources");
			} finally {
				loading = false;
			}
		}
		async function deleteSource(id) {
			deletingId = id;
			try {
				const data = await (await fetch("/api/source", {
					method: "POST",
					headers: { "Content-Type": "application/json" },
					body: JSON.stringify({
						action: "delete",
						id
					})
				})).json();
				if (data.error) throw new Error(data.error);
				addToast("success", "Source deleted");
				await loadSources();
			} catch (e) {
				addToast("error", `Delete failed: ${e.message}`);
			} finally {
				deletingId = null;
			}
		}
		function canReprocess(state) {
			return state === "pending" || state === "error";
		}
		$$renderer.push(`<div class="page svelte-1g9hndx"><h1 class="svelte-1g9hndx">Sources</h1> `);
		if (loading) {
			$$renderer.push("<!--[0-->");
			$$renderer.push(`<div class="card" style="padding: 1rem;"><div class="skeleton" style="height: 1.5rem; width: 6rem; margin-bottom: 1rem;"></div> <!--[-->`);
			const each_array = ensure_array_like([
				1,
				2,
				3,
				4,
				5
			]);
			for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
				each_array[$$index];
				$$renderer.push(`<div class="skeleton" style="height: 1rem; margin-bottom: 0.5rem;"></div>`);
			}
			$$renderer.push(`<!--]--></div>`);
		} else if (sources.length === 0) {
			$$renderer.push("<!--[1-->");
			$$renderer.push(`<div class="card" style="padding: 2rem; text-align: center; color: var(--color-text-muted);"><p>No sources found. Add a source file to <code>.wm/sources/</code> and run <code>source.discover</code>.</p></div>`);
		} else {
			$$renderer.push("<!--[-1-->");
			$$renderer.push(`<div class="card"><div class="table-wrapper svelte-1g9hndx"><table class="svelte-1g9hndx"><thead class="svelte-1g9hndx"><tr class="svelte-1g9hndx"><th class="svelte-1g9hndx">ID</th><th class="svelte-1g9hndx">State</th><th class="svelte-1g9hndx">Pages</th><th class="svelte-1g9hndx">Added</th><th class="svelte-1g9hndx">Actions</th></tr></thead><tbody class="svelte-1g9hndx"><!--[-->`);
			const each_array_1 = ensure_array_like(sources);
			for (let $$index_1 = 0, $$length = each_array_1.length; $$index_1 < $$length; $$index_1++) {
				let src = each_array_1[$$index_1];
				$$renderer.push(`<tr class="svelte-1g9hndx"><td data-label="ID" style="font-family: monospace; font-size: 0.85rem" class="svelte-1g9hndx">${escape_html(src.id)}</td><td data-label="State" class="svelte-1g9hndx"><span${attr_class(`badge state-${stringify(src.state)}`, "svelte-1g9hndx")}>${escape_html(src.state)}</span></td><td data-label="Pages" class="svelte-1g9hndx">${escape_html(src.page_count || 0)}</td><td data-label="Added" style="font-size: 0.85rem; color: var(--color-text-muted)" class="svelte-1g9hndx">${escape_html(src.added_at?.slice(0, 10))}</td><td data-label="Actions" class="svelte-1g9hndx"><div class="action-buttons svelte-1g9hndx">`);
				if (canReprocess(src.state)) {
					$$renderer.push("<!--[0-->");
					$$renderer.push(`<button class="btn-reprocess svelte-1g9hndx"${attr("disabled", processingId === src.id, true)}>${escape_html(processingId === src.id ? "..." : "Reprocess")}</button>`);
				} else $$renderer.push("<!--[-1-->");
				$$renderer.push(`<!--]--> <button class="btn-delete svelte-1g9hndx"${attr("disabled", deletingId === src.id, true)}>${escape_html(deletingId === src.id ? "..." : "Delete")}</button></div></td></tr>`);
			}
			$$renderer.push(`<!--]--></tbody></table></div></div>`);
		}
		$$renderer.push(`<!--]--></div> `);
		if (deleteTarget) {
			$$renderer.push("<!--[0-->");
			ConfirmDialog($$renderer, {
				title: "Delete Source",
				message: "Are you sure you want to delete this source? This action cannot be undone.",
				confirmLabel: "Delete",
				destructive: true,
				busy: deletingId === deleteTarget,
				onConfirm: async () => {
					await deleteSource(deleteTarget);
					deleteTarget = null;
				},
				onCancel: () => deleteTarget = null
			});
		} else $$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]-->`);
	});
}
//#endregion
export { _page as default };
