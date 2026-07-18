import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, ViewChild } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { HlmBadge } from '@ui/badge';
import { HlmCard } from '@ui/card';
import { ApiService } from '../../services/api.service';
import { CanvasGraphDirective } from '@ui/graph';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [HlmBadge, HlmCard, CanvasGraphDirective, WmSpinner, HlmAlert, HlmAlertTitle, HlmAlertDescription],
  changeDetection: ChangeDetectionStrategy.Eager,
  template: `
    <div class="flex flex-col h-full">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Graph</h1>
        @if (stats) {
          <div class="flex items-center gap-3 text-sm text-muted-foreground">
            <span hlmBadge variant="secondary">{{ stats.node_count }} nodes</span>
            <span hlmBadge variant="secondary">{{ stats.edge_count }} edges</span>
          </div>
        }
        <!-- Graph spacing control -->
        <div class="flex items-center gap-2">
          <label class="text-xs text-muted-foreground whitespace-nowrap">Spacing</label>
          <input
            type="range"
            min="20"
            max="200"
            step="10"
            [value]="linkDistance"
            (input)="linkDistance = +$any($event.target).value; restartSim()"
            class="w-20 h-1.5 accent-primary cursor-pointer"
            aria-label="Graph node spacing"
          />
          <span class="text-xs text-muted-foreground w-6 tabular-nums">{{ linkDistance }}</span>
        </div>
      </header>

      <!-- Canvas container -->
      <div class="flex-1 relative bg-muted/30">
        @if (loading) {
          <div class="absolute inset-0 flex items-center justify-center text-muted-foreground text-sm z-10">
            <wm-spinner size="sm" />
            Loading graph...
          </div>
        }
        @if (error) {
          <div class="absolute inset-0 flex items-center justify-center z-10">
            <div hlmAlert variant="destructive" class="max-w-xs text-center shadow-sm">
              <p hlmAlertTitle>Failed to load graph</p>
              <p hlmAlertDescription>{{ error }}</p>
            </div>
          </div>
        }

        <!-- Hover tooltip -->
        @if (hoveredNode) {
          <div
            class="absolute top-2 left-2 z-20 pointer-events-none"
          >
            <div hlmCard class="p-3 text-xs max-w-xs">
              <div class="font-medium text-foreground truncate">{{ hoveredNode.title }}</div>
              <div class="text-muted-foreground font-mono mt-0.5 truncate">{{ hoveredNode.id }}</div>
              <div class="flex items-center gap-2 mt-1.5">
                <span hlmBadge variant="secondary">{{ hoveredNode.page_type }}</span>
                <span class="text-muted-foreground">{{ hoveredNode.degree }} edges</span>
              </div>
            </div>
          </div>
        }
        @if (!loading && !error && graphNodes.length === 0) {
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="p-6 bg-card border border-border rounded-xl shadow-sm text-center max-w-xs">
              <p class="text-muted-foreground font-medium">No graph data</p>
              <p class="text-xs text-muted-foreground/60 mt-1">Create pages with connections to build your wiki graph.</p>
            </div>
          </div>
        }

        <canvas
          wmGraph
          [nodes]="graphNodes"
          [edges]="graphEdges"
          [linkDistance]="linkDistance"
          (nodeClick)="onNodeClick($event)"
          (nodeHover)="onNodeHover($event)"
          (mouseleave)="onMouseLeave()"
          role="img"
          aria-label="Wiki graph visualization"
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
  error = '';
  hoveredNode: any = null;
  linkDistance = 80;
  @ViewChild(CanvasGraphDirective, { static: false }) graphDirective?: CanvasGraphDirective;
  private unlistenFns: Array<() => void> = [];

  constructor(private api: ApiService, private destroyRef: DestroyRef) {}

  ngOnInit() {
    this.api.getGraphFull().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.graphNodes = res.nodes || [];
          this.graphEdges = res.edges || [];
          this.stats = res;
        }
        this.loading = false;
        // Kick off fjadra layout in Tauri mode
        this.startLayout();
      },
      error: () => {
        this.error = 'Failed to load graph data';
        this.loading = false;
      },
    });
  }

  onNodeClick(nodeId: string) {
    console.log('Node clicked:', nodeId);
  }

  restartSim() {
    this.graphDirective?.ngAfterViewInit();
  }

  onNodeHover(node: any) {
    this.hoveredNode = node;
  }

  onMouseLeave() {
    this.hoveredNode = null;
  }

  /** Start fjadra layout via Tauri IPC, listen for position events */
  private startLayout() {
    if (!(window as any).__TAURI_INTERNALS__) return; // browser mode — d3-force handles it

    // Build flat index for node lookup
    const nodeIndex = new Map(this.graphNodes.map((n, i) => [n.id, i]));
    const edges = this.graphEdges
      .map((e: any) => {
        const sId = typeof e.source === 'object' ? e.source.id : e.source;
        const tId = typeof e.target === 'object' ? e.target.id : e.target;
        const s = nodeIndex.get(sId);
        const t = nodeIndex.get(tId);
        return s !== undefined && t !== undefined ? { source: s, target: t } : null;
      })
      .filter(Boolean);

    import('@tauri-apps/api/event').then(async ({ listen }) => {
      // Register all listeners BEFORE firing computeLayout to avoid race
      const unlistenCoarse = await listen<{ positions: [number, number][] }>('graph-coarse', (ev) => {
        this.applyPositions(ev.payload.positions);
      });
      this.unlistenFns.push(unlistenCoarse);

      const unlistenRefine = await listen<{ positions: [number, number][] }>('graph-refine', (ev) => {
        this.applyPositions(ev.payload.positions);
      });
      this.unlistenFns.push(unlistenRefine);

      const unlistenSettled = await listen<{ positions: [number, number][] }>('graph-settled', (ev) => {
        this.applyPositions(ev.payload.positions);
        this.loading = false;
      });
      this.unlistenFns.push(unlistenSettled);

      // All listeners registered — now safe to start layout
      this.api.computeLayout(
        this.graphNodes.map((n: any) => ({ id: n.id })),
        edges,
        window.innerWidth,
        window.innerHeight,
      ).pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
        error: () => {
          this.loading = false;
        },
      });
    });
  }

  /** Apply positions from fjadra to graph nodes */
  private applyPositions(positions: [number, number][]) {
    for (let i = 0; i < positions.length && i < this.graphNodes.length; i++) {
      this.graphNodes[i].x = positions[i][0];
      this.graphNodes[i].y = positions[i][1];
    }
  }
}
