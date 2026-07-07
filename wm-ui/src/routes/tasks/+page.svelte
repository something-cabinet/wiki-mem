<script lang="ts">
	import { onMount } from 'svelte';
	import { addToast } from '$lib/stores/toasts';

	let columns = $state<Record<string, Array<{id: string; title: string; priority: string; assignee?: string}>>>({});
	let counts = $state<Record<string, number>>({});
	let loading = $state(true);
	let quickCreate = $state<Record<string, string>>({ todo: '', in_progress: '', done: '', blocked: '' });
	let creatingStatus = $state<string | null>(null);

	const statusCycle: Record<string, string> = {
		todo: 'in_progress',
		in_progress: 'done',
		done: 'todo',
		blocked: 'todo',
	};

	const statusDisplay: Record<string, string> = {
		todo: 'todo',
		in_progress: 'in-progress',
		done: 'done',
		blocked: 'blocked',
	};

	onMount(async () => {
		await loadBoard();
	});

	async function loadBoard() {
		try {
			const res = await fetch('/api/tasks');
			const data = await res.json();
			columns = data.columns || {};
			counts = data.counts || {};
		} catch (e) {
			addToast('error', 'Failed to load tasks');
		} finally {
			loading = false;
		}
	}

	async function quickAdd(status: string) {
		const title = quickCreate[status].trim();
		if (!title) return;
		creatingStatus = status;
		try {
			const slug = title.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
			const res = await fetch('/api/page', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({
					path: `tasks/${slug}-${Date.now().toString(36)}`,
					title,
					type: 'task',
					content: `---\nstatus: ${status}\n---\n\n`,
				}),
			});
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			quickCreate[status] = '';
			addToast('success', 'Task created');
			await loadBoard();
		} catch (e) {
			addToast('error', 'Failed to create task');
		} finally {
			creatingStatus = null;
		}
	}

	async function cycleStatus(task: { id: string; title: string; priority: string; assignee?: string }, currentStatus: string) {
		const nextStatus = statusCycle[currentStatus] || 'todo';
		try {
			const res = await fetch('/api/tasks', {
				method: 'POST',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ action: 'update', id: task.id, status: statusDisplay[nextStatus] }),
			});
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			addToast('success', `"${task.title}" moved to ${nextStatus.replace('_', ' ')}`);
			await loadBoard();
		} catch (e: any) {
			addToast('error', `Failed to update task: ${e.message}`);
		}
	}
</script>

<div class="page">
	<h1>Task Board</h1>

	{#if loading}
		<div class="board">
			{#each ['todo','in_progress','done','blocked'] as _}
				<div class="column">
					<div class="column-header">
						<div class="skeleton" style="height: 1rem; width: 4rem;"></div>
						<div class="skeleton" style="height: 1rem; width: 1.5rem;"></div>
					</div>
					<div class="cards">
						{#each [1,2] as _}
							<div class="card" style="border-left-color: transparent;">
								<div class="skeleton" style="height: 1rem; margin-bottom: 0.5rem;"></div>
								<div class="skeleton" style="height: 0.75rem; width: 3rem;"></div>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<div class="board">
			{#each ['todo', 'in_progress', 'done', 'blocked'] as status}
				<div class="column">
					<div class="column-header status-{status}">
						<h3>{status.replace('_', ' ')}</h3>
						<span class="count">{counts[status] ?? 0}</span>
					</div>
					<div class="cards">
						{#each (columns[status] || []) as task}
							<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
							<div class="card priority-{task.priority} status-border-{status}" onclick={() => cycleStatus(task, status)} role="button" tabindex="0" aria-label="Task: {task.title}, priority {task.priority}" onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && cycleStatus(task, status)}>
								<div class="card-title">{task.title}</div>
								<div class="card-footer">
									{#if task.assignee}
										<span class="assignee">@{task.assignee}</span>
									{/if}
									<span class="badge priority-{task.priority}">
										{task.priority === 'high' ? '⚠ ' : task.priority === 'medium' ? '● ' : '○ '}{task.priority}
									</span>
									<a href="/page/{task.id}" class="view-link" onclick={(e) => e.stopPropagation()}>View</a>
								</div>
							</div>
						{:else}
							<p class="empty">No tasks</p>
						{/each}
					</div>
					<div class="quick-create">
						<input
							type="text"
							placeholder="Quick create..."
							bind:value={quickCreate[status]}
							onkeydown={(e) => e.key === 'Enter' && quickAdd(status)}
							disabled={creatingStatus === status}
						/>
						<button onclick={() => quickAdd(status)} disabled={creatingStatus === status}>
							{creatingStatus === status ? '...' : '+'}
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.page { padding: 0; }
	h1 { margin-bottom: 1.5rem; }
	.board {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
		gap: 1rem;
	}
	@media (max-width: 640px) {
		.board {
			grid-template-columns: 1fr;
		}
	}
	.column {
		background: var(--color-surface);
		border-radius: var(--radius-md);
		border-top: 3px solid transparent;
		min-height: 200px;
		display: flex;
		flex-direction: column;
	}
	.column-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1rem;
		border-bottom: 1px solid var(--color-border);
		border-top: 3px solid var(--color-gray-light);
		border-radius: var(--radius-md) var(--radius-md) 0 0;
	}
	.column-header.status-todo { border-top-color: var(--color-warning); }
	.column-header.status-in_progress { border-top-color: var(--color-info); }
	.column-header.status-done { border-top-color: var(--color-success); }
	.column-header.status-blocked { border-top-color: var(--color-error); }
	.column-header h3 {
		text-transform: capitalize;
		font-size: 0.9rem;
	}
	.count {
		background: var(--color-border);
		padding: 0.125rem 0.5rem;
		border-radius: 9999px;
		font-size: 0.8rem;
		font-weight: 600;
	}
	.cards {
		padding: 0.5rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		flex: 1;
	}
	.card {
		background: var(--color-bg);
		border: 1px solid var(--color-border);
		border-left: 4px solid transparent;
		border-radius: var(--radius-sm);
		padding: 0.75rem;
		text-decoration: none;
		color: inherit;
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		cursor: pointer;
		transition: box-shadow 0.15s, border-color 0.15s;
	}
	.card:hover {
		border-color: var(--color-primary);
		box-shadow: 0 2px 8px rgba(0,0,0,0.08);
	}
	.card:active {
		transform: scale(0.98);
	}
	.card.priority-high { border-left-color: var(--color-error); }
	.card.priority-medium { border-left-color: var(--color-warning); }
	.card.priority-low { border-left-color: var(--color-info); }
	/* Status border cues */
	.card.status-border-todo { border-bottom: 3px solid var(--color-warning); }
	.card.status-border-in_progress { border-bottom: 3px solid var(--color-info); }
	.card.status-border-done { border-bottom: 3px solid var(--color-success); }
	.card.status-border-blocked { border-bottom: 3px solid var(--color-error); }
	.card-title {
		font-size: 0.875rem;
		font-weight: 500;
	}
	.card-footer {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-top: 0.25rem;
	}
	.assignee {
		font-size: 0.75rem;
		color: var(--color-text-muted);
	}
	.view-link {
		margin-left: auto;
		font-size: 0.75rem;
		color: var(--color-primary);
		text-decoration: none;
		padding: 0.125rem 0.375rem;
		border: 1px solid var(--color-primary-light);
		border-radius: 3px;
	}
	.view-link:hover {
		background: var(--color-primary-light);
		color: var(--color-bg);
	}
	.empty {
		padding: 1rem;
		text-align: center;
		color: var(--color-gray-light);
		font-size: 0.85rem;
	}
	.priority-high { background: var(--color-error-bg); color: var(--color-error); }
	.priority-medium { background: var(--color-warning-bg); color: var(--color-warning); }
	.priority-low { background: var(--color-gray-bg); color: var(--color-gray); }
	.quick-create {
		padding: 0.5rem;
		display: flex;
		gap: 0.5rem;
		border-top: 1px solid var(--color-border);
	}
	.quick-create input {
		flex: 1;
		padding: 0.375rem 0.5rem;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		font-size: 0.875rem;
	}
	.quick-create button {
		padding: 0.375rem 0.75rem;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		background: var(--color-bg);
		font-size: 0.875rem;
		cursor: pointer;
	}
	.quick-create button:hover {
		background: var(--color-surface);
	}
	.quick-create button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
