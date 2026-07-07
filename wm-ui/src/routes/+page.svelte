<script lang="ts">
  import { onMount } from 'svelte';
  import GraphView from '$lib/components/GraphView.svelte';
  import { addToast } from '$lib/stores/toasts';

  let pages = $state<Array<{ id: string; title: string; type: string; status: string }>>([]);
  let stats = $state<{ nodes: number; edges: number; types: Record<string, number> } | null>(null);
  let searchQuery = $state('');
  let searchResults = $state<Array<{ id: string; score: number; snippet: string }>>([]);
  let searchMode = $state('hybrid');
  let activeTab = $state<'pages' | 'graph' | 'search'>('pages');
  let loading = $state(true);

  // Pagination
  let pageNum = $state(1);
  const pageSize = 20;
  let totalPages = $derived(Math.max(1, Math.ceil(pages.length / pageSize)));
  let paginatedPages = $derived(pages.slice((pageNum - 1) * pageSize, pageNum * pageSize));

  function nextPage() { if (pageNum < totalPages) pageNum++; }
  function prevPage() { if (pageNum > 1) pageNum--; }

  // Create page dialog
  let showCreate = $state(false);
  let newPath = $state('');
  let newTitle = $state('');
  let newType = $state('concept');
  let newContent = $state('');
  let creating = $state(false);
  let createError = $state('');

  // Rebuild
  let rebuilding = $state(false);
  let rebuildMsg = $state('');

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      const [pagesRes, graphRes] = await Promise.all([
        fetch('/api/pages'),
        fetch('/api/graph'),
      ]);
      const pagesData = await pagesRes.json();
      const graphData = await graphRes.json();
      pages = pagesData.pages || [];
      stats = { nodes: graphData.nodes || 0, edges: graphData.edges || 0, types: graphData.types || {} };
    } catch (e) {
      addToast('error', 'Failed to load data');
    } finally {
      loading = false;
    }
  }

  async function doSearch() {
    if (!searchQuery.trim()) return;
    const params = new URLSearchParams({ q: searchQuery, mode: searchMode });
    const res = await fetch(`/api/search?${params}`);
    const data = await res.json();
    searchResults = data.results || [];
    activeTab = 'search';
  }

  async function createPage() {
    if (!newPath || !newTitle) return;
    creating = true;
    createError = '';
    try {
      const res = await fetch('/api/page', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path: newPath, title: newTitle, content: newContent, type: newType }),
      });
      const data = await res.json();
      if (data.error) throw new Error(data.error);
      showCreate = false;
      newPath = '';
      newTitle = '';
      newContent = '';
      addToast('success', 'Page created');
      await loadData();
    } catch (e: any) {
      createError = e.message;
      addToast('error', e.message);
    } finally {
      creating = false;
    }
  }

  async function rebuildIndex() {
    rebuilding = true;
    rebuildMsg = '';
    try {
      const res = await fetch('/api/rebuild', { method: 'POST' });
      const data = await res.json();
      rebuildMsg = `Rebuilt: ${data.graph_nodes} nodes, ${data.sections} sections`;
      addToast('success', rebuildMsg);
      await loadData();
    } catch (e: any) {
      rebuildMsg = `Error: ${e.message}`;
      addToast('error', e.message);
    } finally {
      rebuilding = false;
    }
  }

  function typeClass(type: string): string {
    return `badge badge-${type}`;
  }
</script>

{#if loading}
  <div class="stat-grid">
    {#each [1,2,3,4] as _}
      <div class="card stat">
        <div class="skeleton" style="height: 2rem; width: 3rem; margin: 0 auto;"></div>
        <div class="skeleton" style="height: 0.875rem; width: 5rem; margin: 0.5rem auto 0;"></div>
      </div>
    {/each}
  </div>
  <div class="toolbar">
    <div class="search-bar">
      <div class="skeleton" style="height: 2rem; width: 100%;"></div>
      <div class="skeleton" style="height: 2rem; width: 6rem;"></div>
      <div class="skeleton" style="height: 2rem; width: 4rem;"></div>
    </div>
    <div class="actions">
      <div class="skeleton" style="height: 2rem; width: 6rem;"></div>
      <div class="skeleton" style="height: 2rem; width: 7rem;"></div>
    </div>
  </div>
  <div class="tabs skeleton" style="height: 2rem; margin-bottom: 1rem;"></div>
  <div class="card" style="padding: 1rem;">
    {#each [1,2,3,4,5] as _}
      <div class="skeleton" style="height: 1.25rem; margin-bottom: 0.5rem;"></div>
    {/each}
  </div>
{:else}
  <div class="stat-grid">
    <div class="card stat">
      <div class="stat-value">{stats?.nodes ?? 0}</div>
      <div class="stat-label">Wiki Pages</div>
    </div>
    <div class="card stat">
      <div class="stat-value">{stats?.edges ?? 0}</div>
      <div class="stat-label">Relationships</div>
    </div>
    <div class="card stat">
      <div class="stat-value">{Object.keys(stats?.types ?? {}).length}</div>
      <div class="stat-label">Page Types</div>
    </div>
    <div class="card stat">
      <div class="stat-value">{pages.length}</div>
      <div class="stat-label">Total Pages</div>
    </div>
  </div>

  <div class="toolbar">
    <div class="search-bar">
      <input type="text" placeholder="Search wiki..." bind:value={searchQuery}
        onkeydown={(e) => e.key === 'Enter' && doSearch()} />
      <select bind:value={searchMode}>
        <option value="hybrid">Hybrid</option>
        <option value="keyword">Keyword</option>
        <option value="semantic">Semantic</option>
      </select>
      <button onclick={doSearch}>Search</button>
    </div>
    <div class="actions">
      <button class="btn-secondary" onclick={() => showCreate = true}>+ New Page</button>
      <button class="btn-secondary" onclick={rebuildIndex} disabled={rebuilding}>
        {rebuilding ? 'Rebuilding...' : 'Rebuild Index'}
      </button>
    </div>
  </div>

  {#if rebuildMsg}
    <div class="msg">{rebuildMsg}</div>
  {/if}

  <div class="tabs">
    <button class="tab" class:active={activeTab === 'pages'} onclick={() => activeTab = 'pages'}>
      Pages
    </button>
    <button class="tab" class:active={activeTab === 'graph'} onclick={() => activeTab = 'graph'}>
      Graph
    </button>
    <button class="tab" class:active={activeTab === 'search'} onclick={() => activeTab = 'search'}>
      Results ({searchResults.length})
    </button>
  </div>

  {#if activeTab === 'pages'}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="card" onkeydown={(e) => { if (e.key === 'n') { e.preventDefault(); nextPage(); } if (e.key === 'p') { e.preventDefault(); prevPage(); } }}>
      <table>
        <thead><tr><th>Title</th><th>Type</th><th>Status</th><th>ID</th></tr></thead>
        <tbody>
          {#each paginatedPages as page}
            <tr>
              <td><a href="/page/{page.id}" class="page-link">{page.title}</a></td>
              <td><span class={typeClass(page.type)}>{page.type}</span></td>
              <td>{page.status}</td>
              <td style="font-size: 0.8rem; color: var(--color-text-muted); font-family: monospace">{page.id}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if pages.length === 0}
        <p style="padding: 1rem; color: var(--color-text-muted); text-align: center;">No pages yet.</p>
      {:else}
        <div class="pagination">
          <button class="page-btn" onclick={prevPage} disabled={pageNum <= 1}>Prev</button>
          <span class="page-info">{pageNum} / {totalPages}</span>
          <button class="page-btn" onclick={nextPage} disabled={pageNum >= totalPages}>Next</button>
        </div>
      {/if}
    </div>
  {:else if activeTab === 'graph'}
    <GraphView />
  {:else if activeTab === 'search'}
    <div class="card">
      {#if searchResults.length === 0}
        <p style="padding: 1rem; color: var(--color-text-muted); text-align: center;">
          {searchQuery ? 'No results found.' : 'Enter a search query above.'}
        </p>
      {:else}
        <table>
          <thead><tr><th>Score</th><th>ID</th><th>Snippet</th></tr></thead>
          <tbody>
            {#each searchResults as result}
              <tr>
                <td>{(result.score * 100).toFixed(1)}%</td>
                <td style="font-family: monospace; font-size: 0.85rem">
                  <a href="/page/{result.id}" class="page-link">{result.id}</a>
                </td>
                <td>{result.snippet || '-'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}
{/if}

<!-- Create Page Dialog -->
{#if showCreate}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions a11y_interactive_supports_focus -->
  <div class="overlay" role="dialog" aria-modal="true" aria-label="Create page" tabindex="-1" onclick={(e) => { if (e.target === e.currentTarget) showCreate = false; }} onkeydown={(e) => e.key === 'Escape' && (showCreate = false)}>
    <div class="dialog">
      <h2>Create Page</h2>
      <label>Path <input type="text" placeholder="concepts/my-idea" bind:value={newPath} /></label>
      <label>Title <input type="text" placeholder="My Idea" bind:value={newTitle} /></label>
      <label>Type
        <select bind:value={newType}>
          <option value="concept">Concept</option>
          <option value="task">Task</option>
          <option value="spec">Spec</option>
          <option value="decision">Decision</option>
          <option value="pattern">Pattern</option>
          <option value="howto">How-to</option>
          <option value="reference">Reference</option>
        </select>
      </label>
      <label>Content <textarea bind:value={newContent} rows={4}></textarea></label>
      {#if createError}<p class="error">{createError}</p>{/if}
      <div class="dialog-actions">
        <button class="btn-secondary" onclick={() => showCreate = false}>Cancel</button>
        <button onclick={createPage} disabled={creating}>
          {creating ? 'Creating...' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: start;
    gap: 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }
  .search-bar {
    display: flex;
    gap: 0.5rem;
    flex: 1;
    max-width: 500px;
  }
  .search-bar select, .search-bar button {
    padding: 0.5rem;
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    font-size: 0.875rem;
    background: var(--color-bg);
  }
  .search-bar button {
    background: var(--color-primary);
    color: var(--color-bg);
    cursor: pointer;
  }
  .search-bar button:hover { background: var(--color-primary-hover); }
  .actions {
    display: flex;
    gap: 0.5rem;
  }
  .btn-secondary {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
    font-size: 0.875rem;
    cursor: pointer;
  }
  .btn-secondary:hover { background: var(--color-surface); }
  .msg {
    background: var(--color-success-bg);
    color: var(--color-success);
    padding: 0.5rem 1rem;
    border-radius: var(--radius-sm);
    margin-bottom: 1rem;
    font-size: 0.875rem;
  }
  .tabs {
    display: flex;
    gap: 0;
    margin-bottom: 1rem;
    border-bottom: 2px solid var(--color-border);
  }
  .tab {
    padding: 0.5rem 1rem;
    border: none;
    background: none;
    font-size: 0.875rem;
    cursor: pointer;
    color: var(--color-text-muted);
    border-bottom: 2px solid transparent;
    margin-bottom: -2px;
  }
  .tab.active {
    color: var(--color-text);
    border-bottom-color: var(--color-primary);
    font-weight: 600;
  }
  .page-link {
    color: var(--color-primary);
    text-decoration: none;
  }
  .page-link:hover { text-decoration: underline; }
  .pagination {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 0.75rem 0 0.25rem;
    border-top: 1px solid var(--color-border-light);
    margin-top: 0.5rem;
  }
  .page-btn {
    padding: 0.375rem 0.75rem;
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg);
    color: var(--color-text-secondary);
    font-size: 0.8rem;
    cursor: pointer;
  }
  .page-btn:hover:not(:disabled) { background: var(--color-surface); }
  .page-btn:disabled { opacity: 0.4; cursor: default; }
  .page-info {
    font-size: 0.85rem;
    color: var(--color-text-muted);
  }
  .overlay {
    position: fixed;
    inset: 0;
    background: var(--color-overlay);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .dialog {
    background: var(--color-bg);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    width: 480px;
    max-width: 90vw;
  }
  .dialog h2 { margin-bottom: 1rem; }
  .dialog label {
    display: block;
    margin-bottom: 0.75rem;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-secondary);
  }
  .dialog input, .dialog select, .dialog textarea {
    width: 100%;
    margin-top: 0.25rem;
    padding: 0.5rem;
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    font-size: 0.875rem;
    font-family: inherit;
  }
  .dialog textarea { resize: vertical; }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  .dialog-actions button {
    padding: 0.5rem 1rem;
    border: 1px solid var(--color-input-border);
    border-radius: var(--radius-sm);
    background: var(--color-primary);
    color: var(--color-bg);
    font-size: 0.875rem;
    cursor: pointer;
  }
  .dialog-actions button:disabled { opacity: 0.5; }
  .dialog-actions .btn-secondary { background: var(--color-bg); color: var(--color-text-secondary); }
  .error { color: var(--color-error); font-size: 0.85rem; margin-top: 0.5rem; }
</style>
