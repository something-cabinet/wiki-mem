/**
 * Maps Tauri IPC command names to HTTP URL paths used in stub JSON files.
 * This lets us reuse existing mapping files without changes.
 */
export interface CmdMapping {
  method: string;
  urlPath: string;
}

export const CMD_MAP: Record<string, CmdMapping> = {
  search:              { method: 'POST', urlPath: '/api/search' },
  get_initial:         { method: 'POST', urlPath: '/api/initial' },
  list_pages:          { method: 'POST', urlPath: '/api/pages/list' },
  get_page:            { method: 'POST', urlPath: '/api/pages/get' },
  create_page:         { method: 'POST', urlPath: '/api/pages/create' },
  task_board:          { method: 'POST', urlPath: '/api/tasks/board' },
  list_memory:         { method: 'POST', urlPath: '/api/memory/list' },
  get_graph_full:      { method: 'POST', urlPath: '/api/graph/full' },
  get_graph_stats:     { method: 'POST', urlPath: '/api/graph/stats' },
  get_graph_neighbors: { method: 'POST', urlPath: '/api/graph/neighbors' },
};
