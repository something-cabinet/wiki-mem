import { Injectable } from '@angular/core';

export interface RerankDoc { id: string; title: string; body?: string; }
export interface ScoredDoc { id: string; score: number; }

@Injectable({ providedIn: 'root' })
export class Bm25RerankService {
  private wasm: any = null;
  private loaded = false;

  async load(): Promise<void> {
    if (this.loaded) return;
    const wasmModule = await import('../../assets/wasm/bm25-rerank/bm25_rerank_wasm.js');
    await wasmModule.default();
    this.wasm = wasmModule;
    this.loaded = true;
  }

  rerank(query: string, docs: RerankDoc[]): Promise<ScoredDoc[]> {
    if (!this.wasm) return Promise.resolve(docs.map(d => ({ id: d.id, score: 0 })));
    const result = JSON.parse(this.wasm.rerank(query, JSON.stringify(docs)));
    return Promise.resolve(result);
  }
}
