import { y as writable } from "./index-server.js";
import "./index-server2.js";
//#region src/lib/stores/toasts.ts
var toasts = writable([]);
function addToast(type, message) {
	const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
	toasts.update((t) => [...t, {
		id,
		type,
		message
	}]);
	setTimeout(() => {
		toasts.update((t) => t.filter((toast) => toast.id !== id));
	}, 3e3);
}
//#endregion
export { toasts as n, addToast as t };
