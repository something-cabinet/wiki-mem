import { Injectable } from '@angular/core';

export interface GraphAlgoNode { id: string; }
export interface GraphAlgoEdge { source: string; target: string; edgeType?: string; }

@Injectable({ providedIn: 'root' })
export class GraphAlgoService {
  private wasm: any = null;
  private loaded = false;

  async load(): Promise<void> {
    if (this.loaded) return;
    const wasmModule = await import('../../assets/wasm/graph-algo/graph_algo_wasm.js');
    await wasmModule.default();
    this.wasm = wasmModule;
    this.loaded = true;
  }

  createGraph(nodes: GraphAlgoNode[], edges: GraphAlgoEdge[]) {
    return this.wasm.GraphAlgo.new(JSON.stringify(nodes), JSON.stringify(edges));
  }

  findPath(graph: any, startId: string, endId: string): Promise<string[]> {
    const result = JSON.parse(graph.find_path(startId, endId));
    return Promise.resolve(result.ids);
  }

  getNeighbors(graph: any, nodeId: string): Promise<{id: string; edge_type: string}[]> {
    const result = JSON.parse(graph.neighbors(nodeId));
    return Promise.resolve(result);
  }

  getSubgraph(graph: any, centerId: string, depth: number): Promise<{nodes: string[]; edges: any[]}> {
    const result = JSON.parse(graph.subgraph(centerId, depth));
    return Promise.resolve(result);
  }
}
