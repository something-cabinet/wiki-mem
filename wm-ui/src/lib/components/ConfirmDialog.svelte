<script lang="ts">
	import { focusTrap } from '$lib/actions/focusTrap';

	let { title = 'Confirm', message = 'Are you sure?', confirmLabel = 'Confirm', cancelLabel = 'Cancel', destructive = false, busy = false, onConfirm, onCancel }: {
		title?: string;
		message?: string;
		confirmLabel?: string;
		cancelLabel?: string;
		destructive?: boolean;
		busy?: boolean;
		onConfirm: () => void;
		onCancel: () => void;
	} = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
<div class="overlay" role="dialog" aria-modal="true" aria-label={title} tabindex="-1" use:focusTrap
	onclick={(e) => { if (e.target === e.currentTarget) onCancel(); }}
	onkeydown={(e) => e.key === 'Escape' && onCancel()}>
	<div class="dialog">
		<h2>{title}</h2>
		<p class="message">{message}</p>
		<div class="dialog-actions">
			<button class="btn-cancel" onclick={onCancel} disabled={busy}>{cancelLabel}</button>
			<button class="btn-confirm" class:destructive onclick={onConfirm} disabled={busy}>
				{busy ? `${confirmLabel}...` : confirmLabel}
			</button>
		</div>
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
		width: 400px;
		max-width: 90vw;
		box-shadow: 0 8px 32px rgba(0,0,0,0.2);
	}
	.dialog h2 {
		margin-bottom: 0.75rem;
		font-size: 1.1rem;
	}
	.message {
		font-size: 0.9rem;
		color: var(--color-text-muted);
		margin-bottom: 1.25rem;
		line-height: 1.5;
	}
	.dialog-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
	}
	.btn-cancel, .btn-confirm {
		padding: 0.5rem 1rem;
		border-radius: var(--radius-sm);
		font-size: 0.875rem;
		cursor: pointer;
		border: 1px solid var(--color-input-border);
	}
	.btn-cancel {
		background: var(--color-bg);
		color: var(--color-text-secondary);
	}
	.btn-cancel:hover { background: var(--color-surface); }
	.btn-confirm {
		background: var(--color-primary);
		color: var(--color-bg);
		border-color: var(--color-primary);
	}
	.btn-confirm:hover { background: var(--color-primary-hover); }
	.btn-confirm.destructive {
		background: var(--color-error);
		border-color: var(--color-error);
	}
	.btn-confirm.destructive:hover { opacity: 0.9; }
	.btn-confirm:disabled, .btn-cancel:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
