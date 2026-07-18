import fs from 'fs';
import path from 'path';

/** Path to the mock server mappings directory */
const MAPPINGS_DIR = path.resolve(__dirname, '../../wm-web-e2e/mappings');

interface MockMapping {
  request: { method: string; url: string };
  response: { status: number; body: any };
}

/**
 * Load mock server mappings and register them via browser.mockIPC().
 * Maps the mock server's HTTP endpoints to Tauri IPC commands.
 *
 * URL → IPC command mapping:
 *   /api/pages/list    → list_pages
 *   /api/search        → search
 *   /api/task/board    → task_board
 *   /api/graph/full    → get_graph_full
 *   /api/graph/stats   → get_graph_stats
 *   /api/graph/neighbors → get_graph_neighbors
 *   /api/memory/list   → list_memory
 *   /api/initial       → get_initial
 */
export async function loadMockMappings(): Promise<void> {
  const files = fs.readdirSync(MAPPINGS_DIR).filter(f => f.endsWith('.json'));

  for (const file of files) {
    const content = fs.readFileSync(path.join(MAPPINGS_DIR, file), 'utf-8');
    const mapping: MockMapping = JSON.parse(content);
    const cmd = urlToIpcCommand(mapping.request.url);

    if (cmd) {
      await browser.mockIPC(cmd, mapping.response.body);
    }
  }
}

/** Map HTTP URL paths to Tauri IPC command names */
function urlToIpcCommand(url: string): string | null {
  const map: Record<string, string> = {
    '/api/pages/list': 'list_pages',
    '/api/pages/get': 'get_page',
    '/api/pages/create': 'create_page',
    '/api/search': 'search',
    '/api/task/board': 'task_board',
    '/api/graph/full': 'get_graph_full',
    '/api/graph/stats': 'get_graph_stats',
    '/api/graph/neighbors': 'get_graph_neighbors',
    '/api/memory/list': 'list_memory',
    '/api/initial': 'get_initial',
  };
  return map[url] || null;
}

/** Register a single mock IPC handler */
export async function mockIpc(ipcCommand: string, responseData: any): Promise<void> {
  await browser.mockIPC(ipcCommand, responseData);
}
