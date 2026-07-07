<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { addToast } from '$lib/stores/toasts';

	let pid = $state('');
	let title = $state('');
	let type = $state('concept');
	let status = $state('draft');
	let content = $state('');
	let loading = $state(true);
	let saving = $state(false);
	let error = $state('');

	function parseFrontmatter(text: string) {
		const result = { frontmatter: {} as Record<string, string>, body: text };
		if (text.startsWith('---')) {
			const parts = text.split('---');
			if (parts.length >= 3) {
				const yamlText = parts[1].trim();
				result.body = parts.slice(2).join('---').trim();
				yamlText.split('\n').forEach((line) => {
					const match = line.match(/^([^:]+):\s*(.*)$/);
					if (match) {
						result.frontmatter[match[1].trim()] = match[2].trim();
					}
				});
			}
		}
		return result;
	}

	function reconstructContent(titleVal: string, typeVal: string, statusVal: string, body: string): string {
		const fm: string[] = [];
		fm.push('---');
		if (titleVal) fm.push(`title: ${titleVal}`);
		if (typeVal) fm.push(`type: ${typeVal}`);
		if (statusVal) fm.push(`status: ${statusVal}`);
		fm.push('---');
		fm.push('');
		fm.push(body);
		return fm.join('\n');
	}

	onMount(async () => {
		pid = $page.params.id ?? '';
		if (!pid) { error = 'No page ID'; loading = false; return; }
		try {
			const res = await fetch(`/api/page/${encodeURIComponent(pid)}`);
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			const parsed = parseFrontmatter(data.content || '');
			title = parsed.frontmatter.title || data.title || pid;
			type = parsed.frontmatter.type || data.type || 'concept';
			status = parsed.frontmatter.status || data.status || 'draft';
			content = parsed.body || data.content || '';
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});

	async function handleSubmit() {
		saving = true;
		error = '';
		try {
			const fullContent = reconstructContent(title, type, status, content);
			const res = await fetch(`/api/page/${encodeURIComponent(pid)}`, {
				method: 'PUT',
				headers: { 'Content-Type': 'application/json' },
				body: JSON.stringify({ title, type, status, content: fullContent }),
			});
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			addToast('success', 'Page updated');
			goto(`/page/${encodeURIComponent(pid)}`);
		} catch (e: any) {
			error = e.message;
			addToast('error', e.message);
		} finally {
			saving = false;
		}
	}

	function handleCancel() {
		goto(`/page/${encodeURIComponent(pid)}`);
	}
</script>

<div class="page">
	<a href="/page/{encodeURIComponent(pid)}" class="back">&larr; Back to page</a>

	<h1>Edit Page</h1>

	{#if loading}
		<div class="skeleton" style="height: 2rem; width: 12rem; margin-bottom: 1rem;"></div>
		<div class="skeleton" style="height: 2rem; width: 100%; margin-bottom: 1rem;"></div>
		<div class="skeleton" style="height: 2rem; width: 100%; margin-bottom: 1rem;"></div>
		<div class="skeleton" style="height: 300px; width: 100%;"></div>
	{:else if error && !title}
		<div class="error-card">
			<h2>Failed to load page</h2>
			<p>{error}</p>
			<a href="/" class="back" style="margin-top: 1rem;">&larr; Dashboard</a>
		</div>
	{:else}
		<form onsubmit={handleSubmit}>
			<label>
				Title
				<input type="text" bind:value={title} required />
			</label>
			<label>
				Type
				<select bind:value={type}>
					<option value="task">Task</option>
					<option value="spec">Spec</option>
					<option value="concept">Concept</option>
					<option value="pattern">Pattern</option>
					<option value="decision">Decision</option>
					<option value="howto">How-to</option>
					<option value="reference">Reference</option>
				</select>
			</label>
			<label>
				Status
				<select bind:value={status}>
					<option value="todo">Todo</option>
					<option value="in-progress">In Progress</option>
					<option value="done">Done</option>
					<option value="draft">Draft</option>
					<option value="reviewed">Reviewed</option>
					<option value="approved">Approved</option>
				</select>
			</label>
			<label>
				Content (Markdown)
				<textarea bind:value={content} rows={16}></textarea>
			</label>
			{#if error}
				<p class="form-error">{error}</p>
			{/if}
			<div class="form-actions">
				<button type="button" class="btn-cancel" onclick={handleCancel}>Cancel</button>
				<button type="submit" class="btn-save" disabled={saving}>
					{saving ? 'Saving...' : 'Save Changes'}
				</button>
			</div>
		</form>
	{/if}
</div>

<style>
	.page { padding: 0; }
	.back {
		display: inline-block; margin-bottom: 1rem; color: var(--color-primary); text-decoration: none; font-size: 0.875rem;
	}
	.back:hover { text-decoration: underline; }
	h1 { margin-bottom: 1.5rem; }
	form { max-width: 720px; }
	label {
		display: block;
		margin-bottom: 1rem;
		font-size: 0.85rem;
		font-weight: 500;
		color: var(--color-text-secondary);
	}
	label input, label select, label textarea {
		width: 100%;
		margin-top: 0.25rem;
		padding: 0.5rem;
		border: 1px solid var(--color-input-border);
		border-radius: var(--radius-sm);
		font-size: 0.875rem;
		font-family: inherit;
		background: var(--color-bg);
		color: var(--color-text);
	}
	label textarea {
		resize: vertical;
		font-family: 'SF Mono', 'Fira Code', 'Fira Mono', Menlo, Consolas, monospace;
		font-size: 0.825rem;
		line-height: 1.5;
	}
	.form-error {
		color: var(--color-error);
		font-size: 0.85rem;
		margin-bottom: 1rem;
	}
	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.5rem;
		margin-top: 1rem;
	}
	.btn-cancel {
		padding: 0.5rem 1rem;
		border: 1px solid var(--color-input-border);
		border-radius: var(--radius-sm);
		background: var(--color-bg);
		color: var(--color-text-secondary);
		font-size: 0.875rem;
		cursor: pointer;
	}
	.btn-cancel:hover { background: var(--color-surface); }
	.btn-save {
		padding: 0.5rem 1rem;
		border: 1px solid var(--color-primary);
		border-radius: var(--radius-sm);
		background: var(--color-primary);
		color: var(--color-bg);
		font-size: 0.875rem;
		cursor: pointer;
	}
	.btn-save:hover { background: var(--color-primary-hover); }
	.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
	.error-card { padding: 2rem; text-align: center; color: var(--color-text-muted); }
	.error-card h2 { color: var(--color-error); margin-bottom: 0.5rem; }
</style>
