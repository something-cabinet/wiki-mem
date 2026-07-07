<script lang="ts">
	import { toasts, removeToast } from '$lib/stores/toasts';
</script>

<div class="toast-container" aria-live="polite">
	{#each $toasts as toast (toast.id)}
		<div class="toast {toast.type}" role="alert">
			<span>{toast.message}</span>
			<button class="dismiss" onclick={() => removeToast(toast.id)} aria-label="Dismiss">&times;</button>
		</div>
	{/each}
</div>

<style>
	.toast-container {
		position: fixed;
		bottom: 1.5rem;
		right: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		z-index: 200;
		pointer-events: none;
	}
	.toast-container > .toast {
		pointer-events: auto;
	}
	.toast {
		padding: 0.75rem 1rem;
		border-radius: var(--radius-md);
		font-size: 0.875rem;
		font-weight: 500;
		display: flex;
		align-items: center;
		gap: 0.75rem;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
		z-index: 200;
		animation: slideIn 0.25s ease-out;
		max-width: 360px;
	}
	.toast.success {
		background: var(--color-success-bg);
		color: var(--color-success);
		border: 1px solid var(--color-success);
	}
	.toast.error {
		background: var(--color-error-bg);
		color: var(--color-error);
		border: 1px solid var(--color-error);
	}
	.toast.info {
		background: var(--color-info-bg);
		color: var(--color-info);
		border: 1px solid var(--color-info);
	}
	.dismiss {
		background: none;
		border: none;
		font-size: 1.25rem;
		cursor: pointer;
		padding: 0;
		line-height: 1;
		color: inherit;
		opacity: 0.6;
	}
	.dismiss:hover {
		opacity: 1;
	}
	@keyframes slideIn {
		from {
			transform: translateX(100%);
			opacity: 0;
		}
		to {
			transform: translateX(0);
			opacity: 1;
		}
	}
</style>
