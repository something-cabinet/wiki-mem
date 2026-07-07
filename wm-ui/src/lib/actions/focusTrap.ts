export function focusTrap(node: HTMLElement) {
	const focusableSelector = 'a[href], button:not([disabled]), textarea:not([disabled]), input:not([disabled]), select:not([disabled]), [tabindex]:not([tabindex="-1"]), [data-focus-trap]';

	function getFocusableElements(): HTMLElement[] {
		return Array.from(node.querySelectorAll(focusableSelector));
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key !== 'Tab') return;
		const elements = getFocusableElements();
		if (elements.length === 0) return;
		const first = elements[0];
		const last = elements[elements.length - 1];
		if (e.shiftKey && document.activeElement === first) {
			e.preventDefault();
			last.focus();
		} else if (!e.shiftKey && document.activeElement === last) {
			e.preventDefault();
			first.focus();
		}
	}

	node.addEventListener('keydown', handleKeydown);
	// Focus first element on mount
	const first = getFocusableElements()[0];
	if (first) first.focus();

	return {
		destroy() { node.removeEventListener('keydown', handleKeydown); }
	};
}
