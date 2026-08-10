import { Injectable } from '@angular/core';
import { Observable, from } from 'rxjs';
import { EnginePort, InitialState, SearchResult, Page, TaskBoard, MemoryEntry, GraphFullResponse, GraphStats } from './engine-port';

@Injectable({ providedIn: 'root' })
export class HttpEngineService implements EnginePort {
  private base = '/api';
  private token = this.readToken();
  private readToken(): string {
    return (document.querySelector('meta[name="wm-token"]') as HTMLMetaElement | null)?.content ?? '';
  }
  private async httpCall<T>(action: string, body?: unknown): Promise<T> {
    const attempt = (token: string): Promise<Response> =>
      fetch(`${this.base}/${action}`, {
        method: 'POST', headers: { 'Content-Type': 'application/json', 'x-wm-token': token },
        body: body ? JSON.stringify(body) : undefined,
      });

    let res = await attempt(this.token);
    if (res.status === 401) {
      this.token = this.readToken();
      res = await attempt(this.token);
    }
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
}
