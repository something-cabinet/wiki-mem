import { writable } from 'svelte/store';

export type Toast = {
	id: string;
	type: 'success' | 'error' | 'info';
	message: string;
};

export const toasts = writable<Toast[]>([]);

export function addToast(type: 'success' | 'error' | 'info', message: string) {
	const id = Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
	toasts.update((t) => [...t, { id, type, message }]);
	setTimeout(() => {
		toasts.update((t) => t.filter((toast) => toast.id !== id));
	}, 3000);
}

export function removeToast(id: string) {
	toasts.update((t) => t.filter((toast) => toast.id !== id));
}
