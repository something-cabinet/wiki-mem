import { Injectable } from '@angular/core';
import { Observable, of } from 'rxjs';
import { EnginePort, InitialState, SearchResult, Page, TaskBoard, MemoryEntry, GraphFullResponse, GraphStats } from './engine-port';

@Injectable()
export class MockEngineService implements EnginePort {
  getInitial() { return of({ graph_node_count: 0, graph_edge_count: 0, session_memory_count: 0, uptime_secs: 0, stale: false }); }
  searchQuery(q?: string, type?: string, mode?: string, limit?: number) { return of({ results: [] as SearchResult[] }); }
  listPages() { return of({ pages: [] as Page[] }); }
  getPage(id: string) { return of({ id, title: '', type: 'concept', status: 'draft' }); }
  createPage(path: string, title: string, content?: string, type?: string, tags?: string[]) { return of({ success: true }); }
  updatePage(id: string, fields: Record<string, any>) { return of({ success: true }); }
  deletePage(id: string) { return of({ success: true }); }
  getTaskBoard() { return of({ columns: {} as Record<string, any[]>, counts: {} as Record<string, number> }); }
  listMemory(layer?: string, status?: string) { return of({ entries: [] as MemoryEntry[] }); }
  getGraphFull() { return of({ node_count: 0, edge_count: 0, nodes: [], edges: [] }); }
  getGraphStats() { return of({ node_count: 0, edge_count: 0, type_counts: {} }); }
  getGraphNeighbors(id: string) { return of([]); }
  getGraphPath(start: string, end: string) { return of([]); }
  getGraphSubgraph(center: string, depth?: number) { return of([]); }
  rebuildIndex() { return of({ success: true, nodes: 0 }); }
}
