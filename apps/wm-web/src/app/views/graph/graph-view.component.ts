import { Component, OnInit, ChangeDetectionStrategy } from '@angular/core';
import { WmButton } from '@ui/button';
import { WmInput } from '@ui/input';
import { WmBadge } from '@ui/badge';
import { ApiService } from '../../services/api.service';
import { CanvasGraphDirective } from '@ui/graph';

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [WmButton, WmInput, WmBadge, CanvasGraphDirective],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <!-- Header bar -->
      <div class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-bold">Graph</h1>
        @if (stats) {
          <div class="flex items-center gap-3 text-sm text-muted-foreground">
            <span wmBadge variant="secondary">{{ stats.node_count }} nodes</span>
            <span wmBadge variant="secondary">{{ stats.edge_count }} edges</span>
          </div>
        }
      </div>

      <!-- Canvas container -->
      <div class="flex-1 relative bg-muted/30">
        @if (loading) {
          <div class="absolute inset-0 flex items-center justify-center text-muted-foreground text-sm">
            <span class="inline-block w-4 h-4 border-2 border-muted-foreground/30 border-t-muted-foreground rounded-full animate-spin mr-2"></span>
            Loading graph...
          </div>
        }
        <canvas
          wmGraph
          [nodes]="graphNodes"
          [edges]="graphEdges"
          (nodeClick)="onNodeClick($event)"
          class="w-full h-full"
        ></canvas>
      </div>
    </div>
  `,
})
export class GraphViewComponent implements OnInit {
  graphNodes: any[] = [];
  graphEdges: any[] = [];
  stats: any = null;
  loading = true;

  constructor(private api: ApiService) {}

  ngOnInit() {
    this.api.getGraphFull().subscribe((res) => {
      if (res.success) {
        this.graphNodes = res.nodes || [];
        this.graphEdges = res.edges || [];
        this.stats = res;
      }
      this.loading = false;
    });
  }

  onNodeClick(nodeId: string) {
    console.log('Node clicked:', nodeId);
    // Future: open slide-out panel or navigate to pages/:id
  }
}
