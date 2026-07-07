<script lang="ts">
	import { onMount } from 'svelte';

	let { center = '', depth = 2 } = $props();

	let nodes = $state<Array<{ id: string; title: string; type: string; depth: number }>>([]);
	let edges = $state<Array<{ source: string; target: string; type: string }>>([]);
	let loading = $state(true);
	let error = $state('');
	let container = $state<HTMLDivElement | null>(null);
	let network = $state<any>(null);
	let nodesDS = $state<any>(null);
	let edgesDS = $state<any>(null);
	let visReady = $state(false);

	function escapeHtml(str: string): string {
		const div = document.createElement('div');
		div.appendChild(document.createTextNode(str));
		return div.innerHTML;
	}

	function getCssVar(name: string, fallback: string): string {
		if (typeof document === 'undefined') return fallback;
		const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
		return value || fallback;
	}

	function getTypeColors(): Record<string, string> {
		return {
			concept: getCssVar('--color-info', '#3b82f6'),
			task: getCssVar('--color-warning', '#f59e0b'),
			spec: getCssVar('--color-purple', '#8b5cf6'),
			decision: getCssVar('--color-pink', '#ec4899'),
			pattern: getCssVar('--color-success', '#10b981'),
			howto: getCssVar('--color-cyan', '#06b6d4'),
			reference: getCssVar('--color-gray', '#6b7280'),
		};
	}

	function getEdgeColors(): Record<string, string> {
		return {
			extends: getCssVar('--color-info', '#3b82f6'),
			implements: getCssVar('--color-purple', '#8b5cf6'),
			depends_on: getCssVar('--color-error', '#ef4444'),
			relates_to: getCssVar('--color-gray-light', '#9ca3af'),
			supports: getCssVar('--color-success', '#10b981'),
			part_of: getCssVar('--color-cyan', '#06b6d4'),
		};
	}

	let typeColors = $state<Record<string, string>>(getTypeColors());
	let edgeColors = $state<Record<string, string>>(getEdgeColors());

	const nodeLegend = $derived(Object.entries(typeColors));
	const edgeLegend = $derived(Object.entries(edgeColors));

	async function fetchData() {
		loading = true;
		error = '';
		try {
			const params = new URLSearchParams();
			if (center) params.set('center', center);
			params.set('depth', String(depth));
			const res = await fetch(`/api/graph?${params}`);
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			nodes = data.nodes || [];
			edges = data.edges || [];
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		// Dynamic import of vis-network (lazy load)
		const vis = await import('vis-network/standalone');
		await import('vis-network/styles/vis-network.css');
		const { Network } = vis;
		const { DataSet } = vis;
		visReady = true;

		// Initial render check
		$effect(() => {
			if (!container) return;
			typeColors = getTypeColors();
			edgeColors = getEdgeColors();
			const nds = new DataSet([]);
			const eds = new DataSet([]);
			nodesDS = nds;
			edgesDS = eds;
			const textMuted = getCssVar('--color-text-muted', '#6b7280');
			const surface = getCssVar('--color-surface', '#f9fafb');
			const inst = new Network(
				container,
				{ nodes: nds, edges: eds },
				{
					nodes: {
						shape: 'dot',
						size: 16,
						font: { size: 12, color: textMuted },
						borderWidth: 2,
						borderWidthSelected: 3,
						color: { border: surface, background: getCssVar('--color-gray-light', '#9ca3af') },
					},
					edges: {
						width: 2,
						smooth: { enabled: true, type: 'continuous', roundness: 0.5 } as any,
						color: { inherit: false },
					},
					physics: {
						stabilization: false,
						barnesHut: {
							gravitationalConstant: -2000,
							centralGravity: 0.3,
							springLength: 95,
							springConstant: 0.04,
							damping: 0.09,
						},
					},
					interaction: {
						hover: true,
						tooltipDelay: 200,
					},
				}
			);
			inst.on('click', (params: any) => {
				if (params.nodes.length > 0) {
					window.location.href = `/page/${encodeURIComponent(params.nodes[0])}`;
				}
			});
			network = inst;
			const ro = new ResizeObserver(() => {
				inst.redraw();
				inst.fit();
			});
			ro.observe(container);
			return () => {
				ro.disconnect();
				inst.destroy();
				network = null;
				nodesDS = null;
				edgesDS = null;
			};
		});

		// Update data effect
		$effect(() => {
			if (!nodesDS || !edgesDS || !network) return;
			const tc = typeColors;
			const ec = edgeColors;
			nodesDS.clear();
			nodesDS.add(
				nodes.map((n) => ({
					id: n.id,
					label: n.title,
					color: tc[n.type] || getCssVar('--color-gray-light', '#9ca3af'),
					title: `<b>${escapeHtml(n.title)}</b><br>Type: ${escapeHtml(n.type)}`,
				}))
			);
			edgesDS.clear();
			edgesDS.add(
				edges.map((e) => ({
					from: e.source,
					to: e.target,
					title: e.type,
					color: { color: ec[e.type] || getCssVar('--color-gray-light', '#9ca3af'), highlight: ec[e.type] || getCssVar('--color-gray-light', '#9ca3af') },
					arrows: 'to',
				}))
			);
			network.fit();
		});
	});

	// Fetch data when center/depth changes
	$effect(() => {
		center;
		depth;
		fetchData();
	});
</script>

<div class="graph-view">
	{#if loading}
		<div class="skeleton graph-skeleton"></div>
	{:else if error}
		<p class="error">{error}</p>
	{:else if nodes.length === 0}
		<p class="empty">No graph data. Run <code>wm init</code> and create some pages first.</p>
	{:else if !visReady}
		<div class="skeleton graph-skeleton"></div>
	{:else}
		<div class="legend">
			<div class="legend-section">
				<span class="legend-title">Nodes</span>
				{#each nodeLegend as [type, color]}
					<span class="legend-item">
						<span class="legend-dot" style="background: {color}"></span>
						{type}
					</span>
				{/each}
			</div>
			<div class="legend-section">
				<span class="legend-title">Edges</span>
				{#each edgeLegend as [type, color]}
					<span class="legend-item">
						<span class="legend-line" style="background: {color}"></span>
						{type}
					</span>
				{/each}
			</div>
		</div>

		<div class="graph-container" bind:this={container}></div>
	{/if}
</div>

<style>
	.graph-view {
		margin: 1rem 0;
	}
	.graph-skeleton {
		height: 500px;
		width: 100%;
		border-radius: var(--radius-md);
	}
	.error, .empty {
		padding: 2rem;
		text-align: center;
		color: var(--color-text-muted);
	}
	.error {
		color: var(--color-error);
	}
	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: 1.5rem;
		margin-bottom: 1rem;
		font-size: 0.8rem;
		color: var(--color-text);
	}
	.legend-section {
		display: flex;
		flex-wrap: wrap;
		gap: 0.75rem;
		align-items: center;
	}
	.legend-title {
		font-weight: 600;
		margin-right: 0.5rem;
	}
	.legend-item {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}
	.legend-dot {
		width: 10px;
		height: 10px;
		border-radius: 50%;
		display: inline-block;
	}
	.legend-line {
		width: 20px;
		height: 3px;
		border-radius: 2px;
		display: inline-block;
	}
	.graph-container {
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		overflow: hidden;
		background: var(--color-surface);
		height: 500px;
		width: 100%;
	}
</style>
