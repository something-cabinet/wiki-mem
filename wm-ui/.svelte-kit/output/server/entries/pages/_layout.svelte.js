import { c as store_get, g as escape_html, i as attr_class, l as stringify, m as attr, o as ensure_array_like, u as unsubscribe_stores } from "../../chunks/index-server.js";
import { t as page } from "../../chunks/stores.js";
import { n as toasts } from "../../chunks/toasts.js";
//#region src/lib/components/Toast.svelte
function Toast($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		$$renderer.push(`<div class="toast-container svelte-1cpok13" aria-live="polite"><!--[-->`);
		const each_array = ensure_array_like(store_get($$store_subs ??= {}, "$toasts", toasts));
		for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
			let toast = each_array[$$index];
			$$renderer.push(`<div${attr_class(`toast ${stringify(toast.type)}`, "svelte-1cpok13")} role="alert"><span>${escape_html(toast.message)}</span> <button class="dismiss svelte-1cpok13" aria-label="Dismiss">×</button></div>`);
		}
		$$renderer.push(`<!--]--></div>`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}
//#endregion
//#region src/lib/components/HelpOverlay.svelte
function HelpOverlay($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		let { onClose } = $$props;
		const shortcuts = [
			{
				key: "/",
				desc: "Focus search"
			},
			{
				key: "n",
				desc: "Next page"
			},
			{
				key: "p",
				desc: "Previous page"
			},
			{
				key: "?",
				desc: "Toggle this help"
			},
			{
				key: "Escape",
				desc: "Close dialogs / help"
			}
		];
		$$renderer.push(`<div class="overlay svelte-286xo5" role="dialog" aria-modal="true" aria-label="Keyboard shortcuts" tabindex="-1"><div class="dialog svelte-286xo5"><div class="header svelte-286xo5"><h2 class="svelte-286xo5">Keyboard Shortcuts</h2> <button class="close-btn svelte-286xo5" aria-label="Close">×</button></div> <div class="shortcuts svelte-286xo5"><!--[-->`);
		const each_array = ensure_array_like(shortcuts);
		for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
			let sc = each_array[$$index];
			$$renderer.push(`<div class="row svelte-286xo5"><kbd class="svelte-286xo5">${escape_html(sc.key)}</kbd> <span class="svelte-286xo5">${escape_html(sc.desc)}</span></div>`);
		}
		$$renderer.push(`<!--]--></div> <p class="note svelte-286xo5">These shortcuts work globally when no input field is focused.</p></div></div>`);
	});
}
//#endregion
//#region src/routes/+layout.svelte
function _layout($$renderer, $$props) {
	$$renderer.component(($$renderer) => {
		var $$store_subs;
		let { children } = $$props;
		let mobileOpen = false;
		let helpOpen = false;
		const navItems = [
			{
				href: "/",
				label: "Dashboard"
			},
			{
				href: "/tasks",
				label: "Tasks"
			},
			{
				href: "/sources",
				label: "Sources"
			},
			{
				href: "/graph",
				label: "Graph"
			}
		];
		function isActive(href) {
			const path = store_get($$store_subs ??= {}, "$page", page).url.pathname;
			if (href === "/") return path === "/";
			return path.startsWith(href);
		}
		$$renderer.push(`<div class="app svelte-12qhfyh"><nav class="svelte-12qhfyh"><div class="nav-left svelte-12qhfyh"><a href="/" class="brand svelte-12qhfyh">WM Engine</a></div> <button class="hamburger svelte-12qhfyh" aria-label="Toggle navigation"><span class="svelte-12qhfyh"></span> <span class="svelte-12qhfyh"></span> <span class="svelte-12qhfyh"></span></button> <div${attr_class("nav-links svelte-12qhfyh", void 0, { "open": mobileOpen })}><!--[-->`);
		const each_array = ensure_array_like(navItems);
		for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
			let item = each_array[$$index];
			$$renderer.push(`<a${attr("href", item.href)}${attr_class("svelte-12qhfyh", void 0, { "active": isActive(item.href) })}>${escape_html(item.label)}</a>`);
		}
		$$renderer.push(`<!--]--></div> <div class="nav-right svelte-12qhfyh"><button class="theme-toggle svelte-12qhfyh" aria-label="Toggle dark mode">`);
		$$renderer.push("<!--[-1-->");
		$$renderer.push(`<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>`);
		$$renderer.push(`<!--]--></button></div></nav> <main class="svelte-12qhfyh">`);
		children($$renderer);
		$$renderer.push(`<!----></main></div> `);
		Toast($$renderer, {});
		$$renderer.push(`<!----> `);
		if (helpOpen) {
			$$renderer.push("<!--[0-->");
			HelpOverlay($$renderer, { onClose: () => helpOpen = false });
		} else $$renderer.push("<!--[-1-->");
		$$renderer.push(`<!--]-->`);
		if ($$store_subs) unsubscribe_stores($$store_subs);
	});
}
//#endregion
export { _layout as default };
