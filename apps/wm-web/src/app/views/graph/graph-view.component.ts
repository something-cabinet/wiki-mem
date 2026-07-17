import { Component, OnInit, ChangeDetectionStrategy } from '@angular/core';
import { WmBadge } from '@ui/badge';
import { WmCard } from '@ui/card';
import { ApiService } from '../../services/api.service';
import { CanvasGraphDirective } from '@ui/graph';

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [WmBadge, WmCard, CanvasGraphDirective],
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
          <div class="absolute inset-0 flex items-center justify-center text-muted-foreground text-sm z-10">
            <span class="inline-block w-4 h-4 border-2 border-muted-foreground/30 border-t-muted-foreground rounded-full animate-spin mr-2"></span>
            Loading graph...
          </div>
        }

        <!-- Hover tooltip -->
        @if (hoveredNode) {
          <div
            class="absolute top-2 left-2 z-20 pointer-events-none"
          >
            <div wmCard class="p-3 text-xs max-w-xs">
              <div class="font-medium text-foreground truncate">{{ hoveredNode.title }}</div>
              <div class="text-muted-foreground font-mono mt-0.5 truncate">{{ hoveredNode.id }}</div>
              <div class="flex items-center gap-2 mt-1.5">
                <span wmBadge variant="secondary">{{ hoveredNode.page_type }}</span>
                <span class="text-muted-foreground">{{ hoveredNode.degree }} edges</span>
              </div>
            </div>
          </div>
        }

        <canvas
          wmGraph
          [nodes]="graphNodes"
          [edges]="graphEdges"
          (nodeClick)="onNodeClick($event)"
          (nodeHover)="onNodeHover($event)"
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
  hoveredNode: any = null;

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
  }

  onNodeHover(node: any) {
    this.hoveredNode = node;
  }
}
