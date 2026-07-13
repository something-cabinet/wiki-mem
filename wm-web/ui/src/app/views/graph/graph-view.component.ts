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
        <div class="flex gap-4 mb-4 text-sm">
          <span class="px-3 py-1 bg-gray-100 rounded">{{ stats.graph_node_count }} nodes</span>
          <span class="px-3 py-1 bg-gray-100 rounded">{{ stats.graph_edge_count }} edges</span>
        </div>
      }
      <div class="flex gap-2 mb-4">
        <input
          [(ngModel)]="nodeId"
          (keyup.enter)="loadNeighbors()"
          placeholder="Enter page ID..."
          class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <button
          (click)="loadNeighbors()"
          class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
        >
          Explore
        </button>
      </div>
      @if (neighbors.length > 0) {
        <div class="space-y-2">
          <h2 class="font-semibold text-gray-700">Neighbors of <code>{{ nodeId }}</code></h2>
          @for (n of neighbors; track n.id) {
            <div class="p-3 bg-white rounded-lg shadow-sm border border-gray-200 flex items-center justify-between">
              <div>
                <span class="font-medium">{{ n.id }}</span>
                <p class="text-xs text-gray-500">{{ n.title }}</p>
              </div>
              <span class="text-xs px-2 py-0.5 rounded bg-gray-100">{{ n.edge_type }}</span>
            </div>
          }
        </div>
      }
      @if (error) {
        <p class="text-red-500">{{ error }}</p>
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
}
