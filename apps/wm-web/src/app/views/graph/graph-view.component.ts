import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, ViewChild, inject } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { HlmBadge } from '@ui/badge';
import { HlmCard } from '@ui/card';
import { HlmButton } from '@ui/button';
import { ApiService } from '../../services/api.service';
import { CanvasGraphDirective, GraphColorService } from '@ui/graph';
import { WmSpinner } from '@ui/spinner';
import { HlmAlert, HlmAlertTitle, HlmAlertDescription } from '@ui/alert';

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [HlmBadge, HlmCard, HlmButton, CanvasGraphDirective, WmSpinner, HlmAlert, HlmAlertTitle, HlmAlertDescription],
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
      </header>

      <!-- Canvas container -->
      <div class="flex-1 relative bg-muted/30">
        @if (loading) {
          <div class="absolute inset-0 flex items-center justify-center text-muted-foreground text-sm z-10 gap-2">
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
            class="fixed z-20 pointer-events-none"
            [style]="{ left: hoveredNode.clientX + 16 + 'px', top: hoveredNode.clientY - 10 + 'px' }"
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
            <div hlmCard class="p-6 text-center max-w-xs">
              <p class="text-muted-foreground font-medium">No graph data</p>
              <p class="text-xs text-muted-foreground/60 mt-1">Create pages with connections to build your wiki graph.</p>
            </div>
          </div>
        }

          <canvas
            wmGraph
            [nodes]="graphNodes"
            [edges]="graphEdges"
            (nodeClick)="onNodeClick($event)"
            (nodeHover)="onNodeHover($event)"
            (mouseleave)="onMouseLeave()"
            role="application"
            aria-label="Interactive wiki graph. Use the toolbar buttons to zoom and fit to view. Nodes can be dragged and clicked to navigate."
            class="w-full h-full bg-muted/30"
          ></canvas>

          <!-- Color legend -->
          <div class="absolute bottom-3 left-3 z-20">
            @if (showLegend) {
              <div class="bg-popover/95 backdrop-blur border border-border rounded-lg p-3 text-xs shadow-sm max-w-44">
                <div class="font-semibold text-muted-foreground mb-1.5 text-[10px] uppercase tracking-wider">Legend</div>
                @for (type of pageTypes; track type.key) {
                  <div class="flex items-center gap-2 py-0.5">
                    <span class="w-2.5 h-2.5 rounded-full shrink-0" [style]="{ background: type.color }"></span>
                    <span class="truncate">{{ type.label }}</span>
                  </div>
                }
              </div>
            }
            <button hlmBtn variant="ghost" size="xs" (click)="showLegend = !showLegend" class="text-xs text-muted-foreground">
              {{ showLegend ? 'Hide' : 'Legend' }}
            </button>
          </div>

          <!-- Floating toolbar (spacing + zoom) -->
          <div class="absolute bottom-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-2 bg-popover text-popover-foreground border-border rounded-lg px-3 py-1.5 text-xs shadow-sm">
            <button hlmBtn variant="outline" size="icon-xs" (click)="zoomBy(1/1.3)" aria-label="Zoom out" class="size-6">−</button>
            <button hlmBtn variant="outline" size="icon-xs" (click)="zoomBy(1.3)" aria-label="Zoom in" class="size-6">+</button>
            <button hlmBtn variant="outline" size="icon-xs" (click)="fitToView()" aria-label="Fit to view" class="size-6">⤢</button>
            <span class="w-px h-4 bg-border mx-1"></span>
            <label class="text-muted-foreground whitespace-nowrap">Spacing</label>
            <input
              type="range"
              min="50"
              max="400"
              step="10"
              [value]="linkDistance"
              (input)="linkDistance = +$any($event.target).value; onSpacingChange(linkDistance)"
              class="w-20 h-1 accent-primary cursor-pointer"
              aria-label="Graph node spacing"
            />
            <span class="text-muted-foreground w-6 text-right tabular-nums">{{ linkDistance }}</span>
          </div>
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
  hoveredNode: { id: string; title: string; page_type: string; degree: number; clientX: number; clientY: number } | null = null;
  linkDistance = 180;
  showLegend = false;
  pageTypes: { key: string; label: string; color: string }[] = [];
  @ViewChild(CanvasGraphDirective, { static: false }) graphDirective?: CanvasGraphDirective;

  private router = inject(Router);

  private buildPageTypes(): { key: string; label: string; color: string }[] {
    return this.graphColor.allPageTypes();
  }

  constructor(private api: ApiService, private destroyRef: DestroyRef, private graphColor: GraphColorService) {}

  ngOnInit() {
    this.pageTypes = this.buildPageTypes();
    this.api.getGraphFull().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.graphNodes = res.nodes || [];
          this.graphEdges = res.edges || [];
          this.stats = res;
        }
        this.loading = false;
        this.graphDirective?.triggerRender();
        this.startLayout();
      },
      error: () => {
        this.error = 'Failed to load graph data';
        this.loading = false;
      },
    });

    this.graphColor.themeChanged$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.pageTypes = this.buildPageTypes();
        this.graphDirective?.triggerRender();
      });
  }

  onNodeClick(nodeId: string) {
    this.router.navigate(['/pages', nodeId]);
  }

  zoomBy(factor: number) {
    this.graphDirective?.zoomBy(factor);
  }

  fitToView() {
    this.graphDirective?.fitToView();
  }

  onSpacingChange(value: number) {
    this.linkDistance = value;
  }

  onNodeHover(node: any) {
    this.hoveredNode = node;
  }

  onMouseLeave() {
    this.hoveredNode = null;
  }

  /** Start fjadra layout via HTTP SSE, listen for position events */
  private startLayout() {
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

    fetch('http://localhost:4090/api/graph/layout', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        nodes: this.graphNodes.map((n: any) => ({ id: n.id })),
        edges,
        width: window.innerWidth,
        height: window.innerHeight,
        link_distance: this.linkDistance,
      }),
    })
      .then(r => r.json())
      .then((data) => {
        if (data.positions) {
          this.applyPositions(data.positions);
          this.loading = false;
          this.fitToView();
          return;
        }
        const job_id = data.job_id;
        if (!job_id) {
          this.loading = false;
          return;
        }
        const source = new EventSource(`http://localhost:4090/api/graph/layout/${job_id}/events`);
        source.addEventListener('graph-coarse', (e: any) => {
          this.applyPositions(JSON.parse(e.data).positions);
        });
        source.addEventListener('graph-refine', (e: any) => {
          this.applyPositions(JSON.parse(e.data).positions);
        });
        source.addEventListener('graph-settled', (e: any) => {
          this.applyPositions(JSON.parse(e.data).positions);
          this.loading = false;
          this.fitToView();
          source.close();
        });
        source.addEventListener('error', () => {
          this.error = 'Layout streaming failed';
          this.loading = false;
          source.close();
        });
      })
      .catch((err) => {
        this.error = err.message || 'Layout computation failed';
        this.loading = false;
      });
  }

  /** Apply positions from fjadra to graph nodes */
  private applyPositions(positions: [number, number][]) {
    for (let i = 0; i < positions.length && i < this.graphNodes.length; i++) {
      this.graphNodes[i].x = positions[i][0];
      this.graphNodes[i].y = positions[i][1];
    }
    this.graphDirective?.triggerRender();
  }
}
