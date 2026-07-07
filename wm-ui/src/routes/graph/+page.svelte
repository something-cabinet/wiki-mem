<script lang="ts">
	import { onMount } from 'svelte';

	let center = $state('');
	let GraphViewComponent = $state<any>(null);

	onMount(async () => {
		const mod = await import('$lib/components/GraphView.svelte');
		GraphViewComponent = mod.default;
	});
</script>

<div class="page">
	<h1>Graph View</h1>
	<div class="controls">
		<input type="text" placeholder="Center page ID (optional)" bind:value={center} />
		<span class="hint">Leave empty to show all pages at depth 1</span>
	</div>
	{#if GraphViewComponent}
		<GraphViewComponent {center} depth={2} />
	{:else}
		<div class="skeleton" style="height: 500px; border-radius: var(--radius-md);"></div>
	{/if}
</div>

<style>
	.page { padding: 0; }
	h1 { margin-bottom: 1rem; }
	.controls {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}
	.controls input {
		max-width: 400px;
	}
	.hint {
		font-size: 0.8rem;
		color: var(--color-text-muted);
	}
</style>
