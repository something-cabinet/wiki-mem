import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';

export interface SearchResult {
  id: string;
  score: number;
  type: string;
  page_type: string;
  snippet: string;
}

export interface Page {
  id: string;
  title: string;
  type: string;
  status: string;
}

export interface TaskBoard {
  columns: Record<string, TaskBoardItem[]>;
  counts: Record<string, number>;
}

export interface TaskBoardItem {
  id: string;
  title: string;
  priority: string;
}

export interface GraphNeighbor {
  id: string;
  title: string;
  page_type: string;
  edge_type: string;
}

export interface MemoryEntry {
  id: string;
  title: string;
  content: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface InitialState {
  graph_node_count: number;
  graph_edge_count: number;
  session_memory_count: number;
  uptime_secs: number;
  stale: boolean;
}

@Injectable({ providedIn: 'root' })
export class ApiService {
  private baseUrl = '/api';

  constructor(private http: HttpClient) {}

  getInitial(): Observable<any> {
    return this.http.get(`${this.baseUrl}/initial`);
  }

  search(q: string, type?: string, mode?: string, limit?: number): Observable<any> {
    let params = new HttpParams().set('q', q);
    if (type) params = params.set('type', type);
    if (mode) params = params.set('mode', mode);
    if (limit) params = params.set('limit', limit.toString());
    return this.http.get(`${this.baseUrl}/search`, { params });
  }

  listPages(): Observable<any> {
    return this.http.get(`${this.baseUrl}/pages`);
  }

  getPage(id: string): Observable<any> {
    return this.http.get(`${this.baseUrl}/pages/${encodeURIComponent(id)}`);
  }

  createPage(path: string, title: string, content?: string, type?: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/pages`, { path, title, content, type });
  }

  deletePage(id: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/pages/${encodeURIComponent(id)}`);
  }

  getTaskBoard(): Observable<any> {
    return this.http.get(`${this.baseUrl}/tasks/board`);
  }

  getGraphStats(): Observable<any> {
    return this.http.get(`${this.baseUrl}/graph/stats`);
  }

  getGraphNeighbors(id: string): Observable<any> {
    return this.http.get(`${this.baseUrl}/graph/neighbors/${encodeURIComponent(id)}`);
  }

  listMemory(layer?: string): Observable<any> {
    let params = new HttpParams();
    if (layer) params = params.set('layer', layer);
    return this.http.get(`${this.baseUrl}/memory`, { params });
  }
}
