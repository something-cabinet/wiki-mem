<script lang="ts">
	import { focusTrap } from '$lib/actions/focusTrap';

	let { onClose }: { onClose: () => void } = $props();

	const shortcuts = [
		{ key: '/', desc: 'Focus search' },
		{ key: 'n', desc: 'Next page' },
		{ key: 'p', desc: 'Previous page' },
		{ key: '?', desc: 'Toggle this help' },
		{ key: 'Escape', desc: 'Close dialogs / help' },
	];
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
<div class="overlay" role="dialog" aria-modal="true" aria-label="Keyboard shortcuts" tabindex="-1" use:focusTrap
	onclick={(e) => { if (e.target === e.currentTarget) onClose(); }}
	onkeydown={(e) => e.key === 'Escape' && onClose()}>
	<div class="dialog">
		<div class="header">
			<h2>Keyboard Shortcuts</h2>
			<button class="close-btn" onclick={onClose} aria-label="Close">&times;</button>
		</div>
		<div class="shortcuts">
			{#each shortcuts as sc}
				<div class="row">
					<kbd>{sc.key}</kbd>
					<span>{sc.desc}</span>
				</div>
			{/each}
		</div>
		<p class="note">These shortcuts work globally when no input field is focused.</p>
	</div>
</div>

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: var(--color-overlay);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 200;
	}
	.dialog {
		background: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		padding: 1.5rem;
		width: 420px;
		max-width: 90vw;
		box-shadow: 0 8px 32px rgba(0,0,0,0.2);
	}
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}
	.header h2 {
		font-size: 1.1rem;
	}
	.close-btn {
		background: none;
		border: none;
		font-size: 1.5rem;
		cursor: pointer;
		color: var(--color-text-muted);
		line-height: 1;
		padding: 0;
	}
	.close-btn:hover { color: var(--color-text); }
	.shortcuts {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 1rem;
		padding: 0.375rem 0;
	}
	.row kbd {
		display: inline-block;
		min-width: 2rem;
		text-align: center;
		padding: 0.125rem 0.5rem;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		font-family: 'SF Mono', 'Fira Code', monospace;
		font-size: 0.8rem;
		font-weight: 600;
		color: var(--color-text);
	}
	.row span {
		font-size: 0.875rem;
		color: var(--color-text-secondary);
	}
	.note {
		margin-top: 1rem;
		font-size: 0.8rem;
		color: var(--color-text-muted);
		font-style: italic;
	}
</style>
