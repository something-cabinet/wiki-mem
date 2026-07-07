import { u as unsubscribe_stores } from "../../../../chunks/index-server.js";
import { t as goto } from "../../../../chunks/client.js";
import "../../../../chunks/stores.js";
import { t as addToast } from "../../../../chunks/toasts.js";
import "../../../../chunks/navigation.js";
import { t as ConfirmDialog } from "../../../../chunks/ConfirmDialog.js";
import { marked } from "marked";
import "isomorphic-dompurify";
//#region src/routes/page/[id]/+page.svelte
function _page($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		let id = "";
		let deleting = false;
		let showDeleteConfirm = false;
		marked.use({ extensions: [{
			name: "wikilink",
			level: "inline",
			start(src) {
				return src.match(/\[\[/)?.index;
			},
			tokenizer(src) {
				const match = /^\[\[([^\[\]|]+)(?:\|([^\[\]|]+))?\]\]/.exec(src);
				if (match) return {
					type: "wikilink",
					raw: match[0],
					target: match[1],
					text: match[2] || match[1]
				};
			},
			renderer(token) {
				const text = token.text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
				return `<a href="/page/${encodeURIComponent(token.target)}" class="wikilink">${text}</a>`;
			}
		}] });
		async function handleDelete() {
			deleting = true;
			try {
				const data = await (await fetch(`/api/page/${encodeURIComponent(id)}`, { method: "DELETE" })).json();
				if (data.error) throw new Error(data.error);
				addToast("success", "Page deleted");
				goto("/");
			} catch (e) {
				addToast("error", `Failed to delete: ${e.message}`);
			} finally {
				deleting = false;
			}
		}
		$$renderer.push(`<div class="page"><a href="/" class="back svelte-rtsy8">← Dashboard</a> `);
		$$renderer.push("<!--[0-->");
		$$renderer.push(`<div class="skeleton" style="height: 2rem; width: 60%; margin-bottom: 1rem;"></div> <div class="skeleton" style="height: 1rem; width: 8rem; margin-bottom: 1rem;"></div> <div class="skeleton" style="height: 300px; width: 100%; border-radius: var(--radius-md);"></div>`);
		$$renderer.push(`<!--]--></div> `);
		if (showDeleteConfirm) {
			$$renderer.push("<!--[0-->");
			ConfirmDialog($$renderer, {
				title: "Delete Page",
				message: "Are you sure you want to delete this page? This action cannot be undone.",
				confirmLabel: "Delete",
				destructive: true,
				busy: deleting,
				onConfirm: handleDelete,
				onCancel: () => showDeleteConfirm = false
			});
		} else $$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]-->`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}
//#endregion
export { _page as default };
