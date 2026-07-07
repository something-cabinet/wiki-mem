	<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { marked } from 'marked';
	import DOMPurify from 'isomorphic-dompurify';
	import { addToast } from '$lib/stores/toasts';
	import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';

	let content = $state('');
	let id = $state('');
	let loading = $state(true);
	let error = $state('');
	let pageTitle = $state('');
	let pageType = $state('');
	let pageStatus = $state('');
	let createdAt = $state('');
	let updatedAt = $state('');
	let backlinks = $state<Array<{ source: string; type: string }>>([]);
	let backlinkNodes = $state<Array<{ id: string; title: string }>>([]);
	let deleting = $state(false);
	let showDeleteConfirm = $state(false);

	const wikilinkExtension: any = {
		name: 'wikilink',
		level: 'inline',
		start(src: string) { return src.match(/\[\[/)?.index; },
		tokenizer(src: string) {
			const rule = /^\[\[([^\[\]|]+)(?:\|([^\[\]|]+))?\]\]/;
			const match = rule.exec(src);
			if (match) {
				return {
					type: 'wikilink',
					raw: match[0],
					target: match[1],
					text: match[2] || match[1],
				};
			}
		},
		renderer(token: any) {
			const text = token.text
				.replace(/&/g, '&amp;')
				.replace(/</g, '&lt;')
				.replace(/>/g, '&gt;')
				.replace(/"/g, '&quot;');
			return `<a href="/page/${encodeURIComponent(token.target)}" class="wikilink">${text}</a>`;
		}
	};

	marked.use({ extensions: [wikilinkExtension] });

	function renderMarkdown(text: string): string {
		const raw = marked.parse(text, { async: false }) as string;
		return DOMPurify.sanitize(raw, { USE_PROFILES: { html: true } });
	}

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

	onMount(async () => {
		const pid = $page.params.id;
		if (!pid) { error = 'No page ID'; loading = false; return; }
		id = pid;
		try {
			const [pageRes, graphRes] = await Promise.all([
				fetch(`/api/page/${encodeURIComponent(pid)}`),
				fetch(`/api/graph?center=${encodeURIComponent(pid)}&depth=1`),
			]);
			const data = await pageRes.json();
			const graphData = await graphRes.json();
			if (data.error) throw new Error(data.error);
			content = data.content || '';
			const parsed = parseFrontmatter(content);
			pageTitle = parsed.frontmatter.title || data.title || pid;
			pageType = parsed.frontmatter.type || data.type || '';
			pageStatus = parsed.frontmatter.status || data.status || '';
			createdAt = data.created_at || parsed.frontmatter.created_at || '';
			updatedAt = data.updated_at || parsed.frontmatter.updated_at || '';

			const allEdges = graphData.edges || [];
			backlinks = allEdges.filter((e: any) => e.target === pid);
			backlinkNodes = graphData.nodes || [];
		} catch (e: any) {
			error = e.message;
		} finally {
			loading = false;
		}
	});

	function confirmDelete() {
		showDeleteConfirm = true;
	}

	async function handleDelete() {
		deleting = true;
		try {
			const res = await fetch(`/api/page/${encodeURIComponent(id)}`, { method: 'DELETE' });
			const data = await res.json();
			if (data.error) throw new Error(data.error);
			addToast('success', 'Page deleted');
			goto('/');
		} catch (e: any) {
			addToast('error', `Failed to delete: ${e.message}`);
		} finally {
			deleting = false;
		}
	}
</script>

<div class="page">
	<a href="/" class="back">&larr; Dashboard</a>

	{#if loading}
		<div class="skeleton" style="height: 2rem; width: 60%; margin-bottom: 1rem;"></div>
		<div class="skeleton" style="height: 1rem; width: 8rem; margin-bottom: 1rem;"></div>
		<div class="skeleton" style="height: 300px; width: 100%; border-radius: var(--radius-md);"></div>
	{:else if error}
		<div class="error-card">
			<h2>Page not found</h2>
			<p>{error}</p>
		</div>
	{:else}
		<div class="meta">
			<h1>{pageTitle}</h1>
			<div class="badges">
				{#if pageType}<span class="badge badge-{pageType}">{pageType}</span>{/if}
				{#if pageStatus}<span class="badge status-badge">{pageStatus}</span>{/if}
			</div>
			{#if createdAt || updatedAt}
				<div class="timestamps">
					{#if createdAt}<span>Created {new Date(createdAt).toLocaleString()}</span>{/if}
					{#if updatedAt}<span>Updated {new Date(updatedAt).toLocaleString()}</span>{/if}
				</div>
			{/if}
			<div class="actions">
				<a href="/page/{encodeURIComponent(id)}/edit" class="btn-edit">Edit</a>
				<button class="btn-delete" onclick={confirmDelete} disabled={deleting}>
					{deleting ? 'Deleting...' : 'Delete'}
				</button>
			</div>
		</div>
		<div class="content card">
			{@html renderMarkdown(parseFrontmatter(content).body)}
		</div>

		{#if backlinks.length > 0}
			<div class="backlinks card">
				<h2>Backlinks</h2>
				<ul>
					{#each backlinks as edge}
						{@const node = backlinkNodes.find(n => n.id === edge.source)}
						<li>
							<a href="/page/{edge.source}">{node?.title || edge.source}</a>
							<span class="edge-type">({edge.type})</span>
						</li>
					{/each}
				</ul>
			</div>
		{/if}
	{/if}
</div>

{#if showDeleteConfirm}
	<ConfirmDialog
		title="Delete Page"
		message="Are you sure you want to delete this page? This action cannot be undone."
		confirmLabel="Delete"
		destructive={true}
		busy={deleting}
		onConfirm={handleDelete}
		onCancel={() => showDeleteConfirm = false}
	/>
{/if}

<style>
	.back {
		display: inline-block; margin-bottom: 1rem; color: var(--color-primary); text-decoration: none; font-size: 0.875rem;
	}
	.back:hover { text-decoration: underline; }
	.meta h1 { font-size: 1.5rem; color: var(--color-text); margin-bottom: 0.5rem; }
	.badges { display: flex; gap: 0.5rem; margin-bottom: 0.5rem; flex-wrap: wrap; }
	.status-badge { background: var(--color-surface); color: var(--color-text-muted); border: 1px solid var(--color-border); }
	.timestamps { font-size: 0.8rem; color: var(--color-text-muted); margin-bottom: 1rem; display: flex; gap: 1rem; flex-wrap: wrap; }
	.actions { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
	.btn-edit {
		display: inline-block;
		padding: 0.375rem 0.75rem;
		border: 1px solid var(--color-input-border);
		border-radius: var(--radius-sm);
		background: var(--color-bg);
		color: var(--color-text-secondary);
		font-size: 0.8rem;
		text-decoration: none;
		cursor: pointer;
	}
	.btn-edit:hover { background: var(--color-surface); }
	.btn-delete {
		padding: 0.375rem 0.75rem;
		border: 1px solid var(--color-error);
		border-radius: var(--radius-sm);
		background: var(--color-error-bg);
		color: var(--color-error);
		font-size: 0.8rem;
		cursor: pointer;
	}
	.btn-delete:hover { background: var(--color-error); color: var(--color-bg); }
	.btn-delete:disabled { opacity: 0.5; cursor: not-allowed; }
	.content { padding: 1.5rem; line-height: 1.7; font-size: 0.95rem; }
	.content :global(h1) { font-size: 1.3rem; margin: 1rem 0 0.5rem; }
	.content :global(h2) { font-size: 1.15rem; margin: 0.75rem 0 0.5rem; }
	.content :global(h3) { font-size: 1.05rem; margin: 0.5rem 0 0.25rem; }
	.content :global(code) { background: var(--color-surface); padding: 0.125rem 0.375rem; border-radius: 3px; font-size: 0.85rem; }
	.content :global(.wikilink) { color: var(--color-primary); text-decoration: none; border-bottom: 1px dashed var(--color-primary-light); }
	.content :global(p) { margin-bottom: 0.75rem; }
	.error-card { padding: 2rem; text-align: center; color: var(--color-text-muted); }
	.error-card h2 { color: var(--color-error); margin-bottom: 0.5rem; }
	.backlinks { margin-top: 1.5rem; }
	.backlinks h2 { font-size: 1.1rem; margin-bottom: 0.75rem; }
	.backlinks ul { list-style: none; padding: 0; }
	.backlinks li { padding: 0.375rem 0; border-bottom: 1px solid var(--color-border); }
	.backlinks li:last-child { border-bottom: none; }
	.backlinks a { color: var(--color-primary); text-decoration: none; }
	.backlinks a:hover { text-decoration: underline; }
	.edge-type { color: var(--color-text-muted); font-size: 0.85rem; margin-left: 0.25rem; }
</style>
