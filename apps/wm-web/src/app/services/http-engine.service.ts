import { Injectable } from '@angular/core';
import { Observable, from } from 'rxjs';
import { EnginePort, InitialState, SearchResult, Page, TaskBoard, MemoryEntry, GraphFullResponse, GraphStats } from './engine-port';

@Injectable({ providedIn: 'root' })
export class HttpEngineService implements EnginePort {
  private base = 'http://localhost:4090/api';
  private async httpCall<T>(action: string, body?: unknown): Promise<T> {
    const res = await fetch(`${this.base}/${action}`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: body ? JSON.stringify(body) : undefined,
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
  }
  private observe<T>(p: Promise<T>): Observable<T> { return from(p); }

  getInitial() { return this.observe(this.httpCall<InitialState>('initial')); }
  searchQuery(q?: string, type?: string, mode?: string, limit?: number) {
    return this.observe(this.httpCall<{success?: boolean; error?: string; results: SearchResult[]}>('search/query', { q, type, mode, limit }));
  }
  listPages() { return this.observe(this.httpCall<{pages: Page[]}>('pages/list', {})); }
  getPage(id: string) { return this.observe(this.httpCall<Page>('pages/get', { id })); }
  createPage(path: string, title: string, content?: string, type?: string, tags?: string[]) {
    return this.observe(this.httpCall('pages/create', { path, title, content, type, tags }));
  }
  updatePage(id: string, fields: Record<string, any>) {
    return this.observe(this.httpCall('pages/update', { id, ...fields }));
  }
  deletePage(id: string) { return this.observe(this.httpCall('pages/delete', { id })); }
  getTaskBoard() { return this.observe(this.httpCall<TaskBoard>('tasks/board', {})); }
  listMemory(layer?: string, status?: string) {
    return this.observe(this.httpCall<{entries: MemoryEntry[]}>('memory/list', { layer, status }));
  }
  getGraphFull() { return this.observe(this.httpCall<GraphFullResponse>('graph/full', {})); }
  getGraphStats() { return this.observe(this.httpCall<GraphStats>('graph/stats', {})); }
  getGraphNeighbors(id: string) {
    return this.observe(this.httpCall<{id:string;title:string;page_type:string;edge_type:string}[]>('graph/neighbors', { id }));
  }
  getGraphPath(start: string, end: string) { return this.observe(this.httpCall('graph/path', { start, end })); }
  getGraphSubgraph(center: string, depth?: number) {
    return this.observe(this.httpCall('graph/subgraph', { center, depth }));
  }
  rebuildIndex() {
    return this.observe(this.httpCall<{success: boolean; nodes: number}>('index/rebuild', {}));
  }
}
