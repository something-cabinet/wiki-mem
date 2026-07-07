<script lang="ts">
	import { onMount } from 'svelte';
	import { addToast } from '$lib/stores/toasts';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	let sources = $state<Array<any>>([]);
	let loading = $state(true);
	let processingId = $state<string | null>(null);
	let deletingId = $state<string | null>(null);
	let deleteTarget = $state<string | null>(null);

	onMount(async () => {
		await loadSources();
	});

	async function loadSources() {
		try {
			const res = await fetch('/api/sources');
			const data = await res.json();
			sources = data.sources || [];
		} catch {
			addToast('error', 'Failed to load sources');
		} finally {
			loading = false;
		}
	}

	async function reprocessSource(id: string) {
		processingId = id;
		try {
			const res = await fetch('/api/source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ action: 'reprocess', id }),
			});
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			addToast('success', 'Source reprocessed');
			await loadSources();
		} catch (e: any) {
			addToast('error', `Reprocess failed: ${e.message}`);
		} finally {
			processingId = null;
		}
	}

	function confirmDeleteSource(id: string) {
		deleteTarget = id;
	}

	async function deleteSource(id: string) {
		deletingId = id;
		try {
			const res = await fetch('/api/source', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ action: 'delete', id }),
			});
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			addToast('success', 'Source deleted');
			await loadSources();
		} catch (e: any) {
			addToast('error', `Delete failed: ${e.message}`);
		} finally {
			deletingId = null;
		}
	}

	function canReprocess(state: string) {
		return state === 'pending' || state === 'error';
	}
</script>

<div class="page">
	<h1>Sources</h1>

	{#if loading}
		<div class="card" style="padding: 1rem;">
			<div class="skeleton" style="height: 1.5rem; width: 6rem; margin-bottom: 1rem;"></div>
			{#each [1,2,3,4,5] as _}
				<div class="skeleton" style="height: 1rem; margin-bottom: 0.5rem;"></div>
			{/each}
		</div>
	{:else if sources.length === 0}
		<div class="card" style="padding: 2rem; text-align: center; color: var(--color-text-muted);">
			<p>No sources found. Add a source file to <code>.wm/sources/</code> and run <code>source.discover</code>.</p>
		</div>
	{:else}
		<div class="card">
			<div class="table-wrapper">
				<table>
					<thead>
						<tr>
							<th>ID</th>
							<th>State</th>
							<th>Pages</th>
							<th>Added</th>
							<th>Actions</th>
						</tr>
					</thead>
					<tbody>
						{#each sources as src}
							<tr>
								<td data-label="ID" style="font-family: monospace; font-size: 0.85rem">{src.id}</td>
								<td data-label="State">
									<span class="badge state-{src.state}">
										{src.state}
									</span>
								</td>
								<td data-label="Pages">{src.page_count || 0}</td>
								<td data-label="Added" style="font-size: 0.85rem; color: var(--color-text-muted)">{src.added_at?.slice(0, 10)}</td>
								<td data-label="Actions">
									<div class="action-buttons">
										{#if canReprocess(src.state)}
											<button class="btn-reprocess" onclick={() => reprocessSource(src.id)} disabled={processingId === src.id}>
												{processingId === src.id ? '...' : 'Reprocess'}
											</button>
										{/if}
										<button class="btn-delete" onclick={() => confirmDeleteSource(src.id)} disabled={deletingId === src.id}>
											{deletingId === src.id ? '...' : 'Delete'}
										</button>
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</div>
	{/if}
</div>

{#if deleteTarget}
	<ConfirmDialog
		title="Delete Source"
		message="Are you sure you want to delete this source? This action cannot be undone."
		confirmLabel="Delete"
		destructive={true}
		busy={deletingId === deleteTarget}
		onConfirm={async () => { await deleteSource(deleteTarget!); deleteTarget = null; }}
		onCancel={() => deleteTarget = null}
	/>
{/if}

<style>
	.page { padding: 0; }
	h1 { margin-bottom: 1.5rem; }
	.table-wrapper {
		overflow-x: auto;
		-webkit-overflow-scrolling: touch;
	}
	@media (max-width: 640px) {
		.table-wrapper table,
		.table-wrapper thead,
		.table-wrapper tbody,
		.table-wrapper tr,
		.table-wrapper th,
		.table-wrapper td {
			display: block;
		}
		.table-wrapper thead tr {
			position: absolute;
			top: -9999px;
			left: -9999px;
		}
		.table-wrapper tr {
			margin-bottom: 0.75rem;
			border: 1px solid var(--color-border);
			border-radius: var(--radius-sm);
			padding: 0.5rem;
		}
		.table-wrapper td {
			border: none;
			padding: 0.25rem 0.5rem;
			text-align: right;
		}
		.table-wrapper td::before {
			content: attr(data-label);
			float: left;
			font-weight: 600;
			color: var(--color-text-muted);
			font-size: 0.75rem;
			text-transform: uppercase;
			letter-spacing: 0.05em;
		}
		.table-wrapper td:last-child {
			border-bottom: none;
		}
	}
	.state-pending { background: var(--color-warning-bg); color: var(--color-warning); }
	.state-processing { background: var(--color-info-bg); color: var(--color-info); }
	.state-done { background: var(--color-success-bg); color: var(--color-success); }
	.state-error { background: var(--color-error-bg); color: var(--color-error); }
	.state-stale { background: var(--color-gray-bg); color: var(--color-gray); }
	.action-buttons {
		display: flex;
		gap: 0.375rem;
	}
	.btn-reprocess {
		padding: 0.25rem 0.5rem;
		border: 1px solid var(--color-primary);
		border-radius: var(--radius-sm);
		background: var(--color-info-bg);
		color: var(--color-primary);
		font-size: 0.75rem;
		cursor: pointer;
	}
	.btn-reprocess:hover { background: var(--color-primary); color: var(--color-bg); }
	.btn-reprocess:disabled { opacity: 0.5; cursor: not-allowed; }
	.btn-delete {
		padding: 0.25rem 0.5rem;
		border: 1px solid var(--color-error);
		border-radius: var(--radius-sm);
		background: var(--color-error-bg);
		color: var(--color-error);
		font-size: 0.75rem;
		cursor: pointer;
	}
	.btn-delete:hover { background: var(--color-error); color: var(--color-bg); }
	.btn-delete:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
