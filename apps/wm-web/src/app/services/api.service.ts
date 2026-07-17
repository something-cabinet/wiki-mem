import { Injectable, Inject, Optional } from '@angular/core';
import { HttpClient } from '@angular/common/http';
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
  private baseUrl = '/api';
  private isTauri: boolean;
  private invoke: InvokeFn | null = null;

  constructor(private http: HttpClient) {
    this.isTauri = !!(window as any).__TAURI_INTERNALS__;
    if (this.isTauri) {
      // Dynamic import to avoid breaking ng serve builds
      import('@tauri-apps/api/core').then(m => {
        this.invoke = m.invoke as InvokeFn;
      }).catch(() => {
        this.isTauri = false;
      });
    }
  }

  private post<T>(path: string, body: Record<string, any> = {}): Observable<T> {
    if (this.invoke) {
      return from(this.invoke(path.replace('/', ''), body) as Promise<T>);
    }
    return this.http.post<T>(`${this.baseUrl}${path}`, body);
  }

  private tauriCmd<T>(cmd: string, args: Record<string, unknown> = {}): Observable<T> {
    if (this.invoke) {
      return from(this.invoke(cmd, args) as Promise<T>);
    }
    return this.http.post<T>(`${this.baseUrl}/${cmd.replace(/_/g, '/')}`, args);
  }

  getInitial(): Observable<any> {
    return this.tauriCmd('get_initial');
  }

  search(q: string, type?: string, mode?: string, limit?: number): Observable<any> {
    return this.invoke
      ? from(this.invoke('search', { payload: { q, type, mode, limit } }) as Promise<any>)
      : this.post('/search', { q, type, mode, limit });
  }

  listPages(): Observable<any> { return this.post('/pages/list'); }
  getPage(id: string): Observable<any> { return this.post('/pages/get', { id }); }
  createPage(path: string, title: string, content?: string, type?: string): Observable<any> {
    return this.post('/pages/create', { path, title, content, type });
  }
  updatePage(id: string, fields: Record<string, any>): Observable<any> {
    return this.post('/pages/update', { id, ...fields });
  }
  deletePage(id: string): Observable<any> { return this.post('/pages/delete', { id }); }
  getTaskBoard(): Observable<any> { return this.post('/tasks/board'); }

  getGraphFull(): Observable<any> { return this.tauriCmd('get_graph_full'); }
  getGraphStats(): Observable<any> { return this.tauriCmd('get_graph_stats'); }
  getGraphNeighbors(id: string): Observable<any> {
    return this.tauriCmd('get_graph_neighbors', { payload: { id } });
  }

  listMemory(layer?: string, status?: string): Observable<any> {
    return this.post('/memory/list', { layer, status });
  }
}
