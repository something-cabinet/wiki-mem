<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import Toast from '$lib/components/Toast.svelte';
	import HelpOverlay from '$lib/components/HelpOverlay.svelte';

	let { children } = $props();
	let mobileOpen = $state(false);
	let darkMode = $state(false);
	let helpOpen = $state(false);

	const navItems = [
		{ href: '/', label: 'Dashboard' },
		{ href: '/tasks', label: 'Tasks' },
		{ href: '/sources', label: 'Sources' },
		{ href: '/graph', label: 'Graph' },
	];

	function isActive(href: string): boolean {
		const path = $page.url.pathname;
		if (href === '/') return path === '/';
		return path.startsWith(href);
	}

	function toggleDark() {
		darkMode = !darkMode;
		applyTheme(darkMode);
		localStorage.setItem('wm-dark-mode', darkMode ? '1' : '0');
	}

	function applyTheme(dark: boolean) {
		document.documentElement.classList.toggle('dark', dark);
	}

	function handleGlobalKeydown(e: KeyboardEvent) {
		// Ignore when typing in input/textarea/select
		const tag = (e.target as HTMLElement)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return;

		if (e.key === '?') {
			e.preventDefault();
			helpOpen = !helpOpen;
		} else if (e.key === '/') {
			e.preventDefault();
			const searchInput = document.querySelector<HTMLInputElement>('input[type="text"][placeholder*="Search"]');
			searchInput?.focus();
		} else if (e.key === 'n' || e.key === 'p') {
			e.preventDefault();
			const cards = document.querySelectorAll<HTMLElement>('.card, tr');
			if (cards.length === 0) return;
			const current = document.activeElement;
			let idx = Array.from(cards).findIndex(c => c === current);
			if (e.key === 'n') {
				idx = (idx + 1) % cards.length;
			} else {
				idx = (idx - 1 + cards.length) % cards.length;
			}
			(cards[idx] as HTMLElement)?.focus();
		}
	}

	onMount(() => {
		const stored = localStorage.getItem('wm-dark-mode');
		if (stored !== null) {
			darkMode = stored === '1';
		} else {
			darkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
		}
		applyTheme(darkMode);

		const mq = window.matchMedia('(prefers-color-scheme: dark)');
		const listener = (e: MediaQueryListEvent) => {
			if (localStorage.getItem('wm-dark-mode') === null) {
				darkMode = e.matches;
				applyTheme(darkMode);
			}
		};
		mq.addEventListener('change', listener);

		document.addEventListener('keydown', handleGlobalKeydown);
		return () => {
			mq.removeEventListener('change', listener);
			document.removeEventListener('keydown', handleGlobalKeydown);
		};
	});
</script>

<div class="app">
	<nav>
		<div class="nav-left">
			<a href="/" class="brand">WM Engine</a>
		</div>
		<button class="hamburger" onclick={() => mobileOpen = !mobileOpen} aria-label="Toggle navigation">
			<span></span>
			<span></span>
			<span></span>
		</button>
		<div class="nav-links" class:open={mobileOpen}>
			{#each navItems as item}
				<a href={item.href} class:active={isActive(item.href)} onclick={() => mobileOpen = false}>
					{item.label}
				</a>
			{/each}
		</div>
		<div class="nav-right">
			<button class="theme-toggle" onclick={toggleDark} aria-label="Toggle dark mode">
				{#if darkMode}
					<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
				{/if}
			</button>
		</div>
	</nav>
	<main>
		{@render children()}
	</main>
</div>

<Toast />

{#if helpOpen}
	<HelpOverlay onClose={() => helpOpen = false} />
{/if}

<style>
	.app {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
	}
	nav {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1.5rem;
		border-bottom: 1px solid var(--color-border);
		background: var(--color-surface);
		position: relative;
	}
	.nav-left {
		display: flex;
		align-items: center;
		gap: 0.75rem;
	}
	.nav-right {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}
	.brand {
		font-weight: 700;
		font-size: 1.1rem;
		color: var(--color-text);
		text-decoration: none;
	}
	.nav-links {
		display: flex;
		gap: 1.5rem;
	}
	.nav-links a {
		color: var(--color-text-muted);
		text-decoration: none;
		font-size: 0.875rem;
		font-weight: 500;
		padding: 0.25rem 0;
		border-bottom: 2px solid transparent;
		transition: color 0.2s, border-color 0.2s;
	}
	.nav-links a:hover {
		color: var(--color-text);
	}
	.nav-links a.active {
		color: var(--color-primary);
		border-bottom-color: var(--color-primary);
	}
	.hamburger {
		display: none;
		flex-direction: column;
		gap: 4px;
		background: none;
		border: none;
		cursor: pointer;
		padding: 0.25rem;
	}
	.hamburger span {
		display: block;
		width: 20px;
		height: 2px;
		background: var(--color-text);
		border-radius: 2px;
	}
	.theme-toggle {
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		cursor: pointer;
		padding: 0.375rem;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted);
		transition: color 0.2s, border-color 0.2s;
	}
	.theme-toggle:hover {
		color: var(--color-text);
		border-color: var(--color-text-muted);
	}
	main {
		flex: 1;
		padding: 1.5rem;
		max-width: 1200px;
		width: 100%;
		margin: 0 auto;
	}
	@media (max-width: 767px) {
		.hamburger {
			display: flex;
		}
		.nav-links {
			display: none;
			position: absolute;
			top: 100%;
			left: 0;
			right: 0;
			flex-direction: column;
			background: var(--color-surface);
			border-bottom: 1px solid var(--color-border);
			padding: 0.5rem 1.5rem;
			gap: 0;
			z-index: 50;
		}
		.nav-links.open {
			display: flex;
		}
		.nav-links a {
			padding: 0.75rem 0;
			border-bottom: 1px solid var(--color-border);
		}
		.nav-links a:last-child {
			border-bottom: none;
		}
		.nav-links a.active {
			border-bottom-color: var(--color-primary);
		}
		main {
			padding: 1rem;
		}
		.nav-right {
			margin-left: auto;
		}
	}
</style>
