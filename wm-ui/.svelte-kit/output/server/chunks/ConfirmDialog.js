import { g as escape_html, i as attr_class, m as attr } from "./index-server.js";
//#region src/lib/components/ConfirmDialog.svelte
function ConfirmDialog($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let { title = "Confirm", message = "Are you sure?", confirmLabel = "Confirm", cancelLabel = "Cancel", destructive = false, busy = false, onConfirm, onCancel } = $$props;
		$$renderer.push(`<div class="overlay svelte-7e0w24" role="dialog" aria-modal="true"${attr("aria-label", title)} tabindex="-1"><div class="dialog svelte-7e0w24"><h2 class="svelte-7e0w24">${escape_html(title)}</h2> <p class="message svelte-7e0w24">${escape_html(message)}</p> <div class="dialog-actions svelte-7e0w24"><button class="btn-cancel svelte-7e0w24"${attr("disabled", busy, true)}>${escape_html(cancelLabel)}</button> <button${attr_class("btn-confirm svelte-7e0w24", void 0, { "destructive": destructive })}${attr("disabled", busy, true)}>${escape_html(busy ? `${confirmLabel}...` : confirmLabel)}</button></div></div></div>`);
	});
}
//#endregion
export { ConfirmDialog as t };
