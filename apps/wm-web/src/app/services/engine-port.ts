import { InjectionToken } from '@angular/core';
import { Observable } from 'rxjs';

export interface GraphFullResponse {
  success?: boolean;
  error?: string;
  node_count: number;
  edge_count: number;
  nodes: { id: string; title: string; page_type: string; degree: number }[];
  edges: { source: string; target: string; edge_type: string }[];
}

export interface ScoreBreakdown {
  bm25: number; rrf: number; semantic: number; title_density: number;
  exact_title: number; tag_overlap: number; exact_id: number;
  recency: number; final_score: number;
}

export interface SearchResult {
  id: string; score: number; type: string; page_type: string; snippet: string;
  score_breakdown?: ScoreBreakdown; showBreakdown?: boolean;
}

export interface Page { id: string; title: string; type: string; status: string; content?: string; success?: boolean; error?: string; }
export interface TaskBoard { success?: boolean; error?: string; columns: Record<string, TaskBoardItem[]>; counts: Record<string, number>; }
export interface TaskBoardItem { id: string; title: string; priority: string; }
export interface GraphNeighbor { id: string; title: string; page_type: string; edge_type: string; }
export interface MemoryEntry { id: string; title: string; content: string; tags: string[]; created_at: string; updated_at: string; }
export interface InitialState { graph_node_count: number; graph_edge_count: number; session_memory_count: number; uptime_secs: number; stale: boolean; }
export interface GraphStats { node_count: number; edge_count: number; type_counts: Record<string, number>; }

export interface EnginePort {
  getInitial(): Observable<InitialState>;
  searchQuery(q: string, type?: string, mode?: string, limit?: number): Observable<{success?: boolean; error?: string; results: SearchResult[]}>;
  listPages(): Observable<{pages: Page[]}>;
  getPage(id: string): Observable<Page>;
  createPage(path: string, title: string, content?: string, type?: string, tags?: string[]): Observable<any>;
  updatePage(id: string, fields: Record<string, any>): Observable<any>;
  deletePage(id: string): Observable<any>;
  getTaskBoard(): Observable<TaskBoard>;
  listMemory(layer?: string, status?: string): Observable<{entries: MemoryEntry[]}>;
  getGraphFull(): Observable<GraphFullResponse>;
  getGraphStats(): Observable<GraphStats>;
  getGraphNeighbors(id: string): Observable<{id:string;title:string;page_type:string;edge_type:string}[]>;
  getGraphPath(start: string, end: string): Observable<any>;
  getGraphSubgraph(center: string, depth?: number): Observable<any>;
  rebuildIndex(): Observable<{success: boolean; nodes: number}>;
}

export const ENGINE_PORT = new InjectionToken<EnginePort>('ENGINE_PORT');
