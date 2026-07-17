import { expect } from '@wdio/globals';

describe('WM Wiki — Graph View', () => {
  before(async () => {
    // Register IPC mocks for all commands
    await browser.mockIPC('get_graph_full', () => ({
      success: true,
      node_count: 10,
      edge_count: 5,
      nodes: [
        { id: 'wiki:test:a', title: 'Node A', page_type: 'concept', degree: 3 },
        { id: 'wiki:test:b', title: 'Node B', page_type: 'task', degree: 2 },
      ],
      edges: [
        { source: 'wiki:test:a', target: 'wiki:test:b', edge_type: 'depends_on' },
      ],
    }));
    await browser.mockIPC('get_graph_stats', () => ({
      success: true, node_count: 10, edge_count: 5,
    }));
  });

  it('should load the graph view with canvas', async () => {
    await browser.url('http://localhost:4200/graph');
    await browser.pause(2000);

    const canvas = await browser.$('canvas[wmGraph]');
    await expect(canvas).toBeExisting();

    const heading = await browser.$('h1');
    await expect(heading).toHaveText('Graph');
  });

  it('should show node and edge counts', async () => {
    await browser.url('http://localhost:4200/graph');
    await browser.pause(2000);

    const badges = await browser.$$('[wmBadge]');
    expect(badges.length).toBeGreaterThanOrEqual(2);
  });
});
