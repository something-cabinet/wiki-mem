import { Component, OnInit } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { ApiService, GraphNeighbor } from '../../services/api.service';

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [FormsModule],
  template: `
    <div class="p-6 max-w-4xl mx-auto">
      <h1 class="text-2xl font-bold mb-4">Graph</h1>
      @if (stats) {
        <div class="grid grid-cols-2 gap-4 mb-6">
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
            <div class="flex items-center justify-between mb-3">
              <span class="text-sm font-medium text-gray-500">Nodes</span>
              <span class="text-2xl font-bold text-slate-800">{{ stats.graph_node_count }}</span>
            </div>
            <div class="flex flex-wrap gap-1">
              @for (i of dotGrid(stats.graph_node_count, 24); track $index) {
                <div class="w-2 h-2 rounded-full bg-blue-400"></div>
              }
            </div>
          </div>
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4">
            <div class="flex items-center justify-between mb-3">
              <span class="text-sm font-medium text-gray-500">Edges</span>
              <span class="text-2xl font-bold text-slate-800">{{ stats.graph_edge_count }}</span>
            </div>
            <div class="flex flex-wrap gap-1">
              @for (i of dotGrid(stats.graph_edge_count, 24); track $index) {
                <div class="w-2 h-2 rounded-full bg-emerald-400"></div>
              }
            </div>
          </div>
        </div>
      }
      <div class="flex gap-2 mb-4">
        <input
          [(ngModel)]="nodeId"
          (keyup.enter)="loadNeighbors()"
          placeholder="Enter page ID..."
          class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-shadow"
        />
        <button
          (click)="loadNeighbors()"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 font-medium transition-colors"
        >
          Explore
        </button>
      </div>
      @if (neighbors.length > 0) {
        <div class="space-y-3">
          <h2 class="font-semibold text-gray-700 text-sm uppercase tracking-wider">
            Neighbors of <code class="px-1.5 py-0.5 bg-slate-100 rounded text-slate-700 text-xs font-mono">{{ nodeId }}</code>
          </h2>
          <div class="grid gap-2">
            @for (n of neighbors; track n.id) {
              <div class="p-3 bg-white rounded-lg shadow-sm border border-gray-200 flex items-center justify-between hover:border-blue-300 transition-colors">
                <div class="min-w-0">
                  <span class="font-medium text-sm block truncate">{{ n.id }}</span>
                  <p class="text-xs text-gray-500 truncate">{{ n.title }}</p>
                </div>
                <div class="flex items-center gap-2 ml-4 shrink-0">
                  <span class="text-xs px-2 py-0.5 rounded-full bg-slate-100 text-slate-600 font-medium">{{ n.page_type }}</span>
                  <span class="text-xs px-2 py-0.5 rounded-full bg-blue-50 text-blue-700 font-medium">{{ n.edge_type }}</span>
                </div>
              </div>
            }
          </div>
        </div>
      }
      @if (error) {
        <p class="text-red-500 text-sm mt-2">{{ error }}</p>
      }
    </div>
  `,
})
export class GraphViewComponent implements OnInit {
  nodeId = '';
  neighbors: GraphNeighbor[] = [];
  stats: any = null;
  error = '';

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.api.getGraphStats().subscribe((res) => {
      if (res.success) this.stats = res;
    });
  }

  loadNeighbors() {
    if (!this.nodeId.trim()) return;
    this.error = '';
    this.api.getGraphNeighbors(this.nodeId).subscribe({
      next: (res) => {
        if (res.success) this.neighbors = res.neighbors || [];
        else this.error = res.error || 'Not found';
      },
      error: () => (this.error = 'Failed to load neighbors'),
    });
  }

  dotGrid(count: number, max: number): number[] {
    const n = Math.min(count, max);
    return Array.from({ length: n }, (_, i) => i);
  }
}
