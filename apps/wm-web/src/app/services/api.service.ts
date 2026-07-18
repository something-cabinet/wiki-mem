import { Injectable } from '@angular/core';
import { Observable, from } from 'rxjs';

export interface SearchResult {
  id: string; score: number; type: string; page_type: string; snippet: string;
}
export interface Page {
  id: string; title: string; type: string; status: string;
}
export interface TaskBoard {
  columns: Record<string, TaskBoardItem[]>; counts: Record<string, number>;
}
export interface TaskBoardItem {
  id: string; title: string; priority: string;
}
export interface GraphNeighbor {
  id: string; title: string; page_type: string; edge_type: string;
}
export interface MemoryEntry {
  id: string; title: string; content: string; tags: string[];
  created_at: string; updated_at: string;
}
export interface InitialState {
  graph_node_count: number; graph_edge_count: number;
  session_memory_count: number; uptime_secs: number; stale: boolean;
}

type InvokeFn = (cmd: string, args?: Record<string, unknown>) => Promise<unknown>;

@Injectable({ providedIn: 'root' })
export class ApiService {
  private invoke: InvokeFn;

  constructor() {
    const wi = window as any;
    if (wi.__MOCK_INVOKE__) {
      this.invoke = wi.__MOCK_INVOKE__;
    } else {
      // Tauri v2 exposes invoke directly on __TAURI_INTERNALS__
      this.invoke = wi.__TAURI_INTERNALS__.invoke as InvokeFn;
    }
  }

  private tauriCmd<T>(cmd: string, args: Record<string, unknown> = {}): Observable<T> {
    return from(this.invoke(cmd, args) as Promise<T>);
  }

  getInitial(): Observable<any> { return this.tauriCmd('get_initial'); }
  search(q: string, type?: string, mode?: string, limit?: number): Observable<any> {
    return this.tauriCmd('search', { payload: { q, type, mode, limit } });
  }
  listPages(): Observable<any> { return this.tauriCmd('list_pages'); }
  getPage(id: string): Observable<any> { return this.tauriCmd('get_page', { payload: { id } }); }
  createPage(path: string, title: string, content?: string, type?: string, tags?: string): Observable<any> {
    return this.tauriCmd('create_page', { payload: { path, title, content, type, tags } });
  }
  updatePage(id: string, fields: Record<string, any>): Observable<any> {
    return this.tauriCmd('update_page', { payload: { id, ...fields } });
  }
  deletePage(id: string): Observable<any> {
    return this.tauriCmd('delete_page', { payload: { id } });
  }
  getTaskBoard(): Observable<any> { return this.tauriCmd('task_board'); }
  listMemory(layer?: string, status?: string): Observable<any> {
    return this.tauriCmd('list_memory', { payload: { _layer: layer, _status: status } });
  }
  getGraphFull(): Observable<any> { return this.tauriCmd('get_graph_full'); }
  getGraphStats(): Observable<any> { return this.tauriCmd('get_graph_stats'); }
  computeLayout(nodes: any[], edges: any[], width: number, height: number): Observable<any> {
    return this.tauriCmd('compute_layout', { payload: { nodes, edges, width, height } });
  }
  getGraphNeighbors(id: string): Observable<any> {
    return this.tauriCmd('get_graph_neighbors', { payload: { id } });
  }
}
