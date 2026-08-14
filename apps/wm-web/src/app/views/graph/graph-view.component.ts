import { Component, OnInit, ChangeDetectionStrategy, DestroyRef, ViewChild, Inject, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { Router } from '@angular/router';
import { HlmBadge } from '@ui/badge';
import { HlmCard } from '@ui/card';
import { HlmButton } from '@ui/button';
import { EnginePort, ENGINE_PORT } from '../../services/engine-port';
import { CanvasGraphDirective, GraphColorService } from '@ui/graph';
import { WmSpinner } from '@ui/spinner';
import { WmSkeleton } from '@ui/skeleton';
import { WmErrorState } from '../../components/error-state/error-state.component';

/**
 * Maps the spacing slider value (50–400) to fjadra's global many-body
 * repulsion. A stronger negative charge pushes ALL nodes apart (not just
 * linked pairs). Default slider value 180 → -200 (≈ the pre-slider constant).
 */
const SLIDER_TO_CHARGE = (value: number): number => -Math.round(value * 1.1111);
/** Debounce delay (ms) before the layout recomputes after a slider change. */
const SPACING_RECOMPUTE_MS = 150;

interface PageTypeEntry {
  key: string;
  label: string;
  color: string;
}

interface ProvenanceEntry {
  key: string;
  label: string;
  alpha: number;
  dash: number[];
}

interface HoveredNode {
  id: string;
  title: string;
  page_type: string;
  degree: number;
  clientX: number;
  clientY: number;
}

@Component({
  selector: 'app-graph-view',
  standalone: true,
  imports: [HlmBadge, HlmCard, HlmButton, CanvasGraphDirective, WmSpinner, WmSkeleton, WmErrorState],
  changeDetection: ChangeDetectionStrategy.OnPush,
  template: `
    <div class="flex flex-col h-full wm-page-enter">
      <header class="flex items-center justify-between px-6 py-3 border-b border-border bg-card shrink-0">
        <h1 class="text-xl sm:text-2xl font-semibold">Graph</h1>
        @if (stats(); as stats) {
          <div class="flex items-center gap-3 text-sm text-muted-foreground">
            <span hlmBadge variant="secondary">{{ stats.node_count }} nodes</span>
            <span hlmBadge variant="secondary">{{ stats.edge_count }} edges</span>
          </div>
        }
      </header>

      <div class="flex-1 relative bg-muted/30">
        @if (loading()) {
          <div
            class="absolute inset-0 flex items-center justify-center z-30"
            role="status"
            aria-live="polite"
            aria-busy="true"
          >
            <div class="flex flex-col items-center gap-6">
              <div class="flex items-center gap-2 text-muted-foreground text-sm">
                <wm-spinner size="sm" />
                Loading graph...
              </div>
              <div class="relative h-40 w-72" aria-hidden="true">
                <wm-skeleton class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 size-20 rounded-full" />
                <wm-skeleton class="absolute left-0 top-1/3 size-8 rounded-full" />
                <wm-skeleton class="absolute right-4 top-1/4 size-6 rounded-full" />
                <wm-skeleton class="absolute left-10 bottom-2 size-9 rounded-full" />
                <wm-skeleton class="absolute right-10 bottom-6 size-5 rounded-full" />
                <wm-skeleton class="absolute top-0 right-1/3 size-7 rounded-full" />
              </div>
            </div>
          </div>
        }
        @if (error()) {
          <div class="absolute inset-0 flex items-center justify-center z-30">
            <wm-error-state title="Failed to load graph" [message]="error()" (retry)="reload()" />
          </div>
        }

        @if (hoveredNode(); as hoveredNode) {
          <div
            class="fixed z-20 pointer-events-none"
            [style]="{ left: tooltipX + 'px', top: tooltipY + 'px' }"
          >
            <div hlmCard class="p-3 text-xs max-w-xs shadow-lg">
              <div class="font-medium text-foreground truncate">{{ hoveredNode.title }}</div>
              <div class="text-muted-foreground font-mono mt-0.5 truncate">{{ hoveredNode.id }}</div>
              <div class="flex items-center gap-2 mt-1.5">
                <span hlmBadge variant="secondary">{{ hoveredNode.page_type }}</span>
                <span class="text-muted-foreground">{{ hoveredNode.degree }} edges</span>
              </div>
            </div>
          </div>
        }
        @if (!loading() && !error() && graphNodes().length === 0) {
          <div class="absolute inset-0 flex items-center justify-center z-10">
            <div hlmCard class="p-6 text-center max-w-xs">
              <p class="text-muted-foreground font-medium">No graph data</p>
              <p class="text-xs text-muted-foreground/60 mt-1">Create pages with connections to build your wiki graph.</p>
            </div>
          </div>
        }

          <canvas
            wmGraph
            [nodes]="graphNodes()"
            [edges]="graphEdges()"
            (nodeClick)="onNodeClick($event)"
            (nodeHover)="onNodeHover($event)"
            (mouseleave)="onMouseLeave()"
            role="application"
            aria-label="Interactive wiki graph. Use the toolbar buttons to zoom and fit to view. Nodes can be dragged and clicked to navigate."
            class="w-full h-full bg-muted/30"
          ></canvas>

          <div class="absolute bottom-3 left-3 z-20">
            @if (showLegend()) {
              <div class="bg-popover/95 backdrop-blur border border-border rounded-lg p-3 text-xs shadow-sm max-w-44">
                <div class="font-semibold text-muted-foreground mb-1.5 text-[10px] uppercase tracking-wider">Legend</div>
                @for (type of pageTypes(); track type.key) {
                  <div class="flex items-center gap-2 py-0.5">
                    <span class="w-2.5 h-2.5 rounded-full shrink-0" [style]="{ background: type.color }"></span>
                    <span class="truncate">{{ type.label }}</span>
                  </div>
                }
                <div class="border-t border-border my-1.5"></div>
                @for (entry of provenanceLegend; track entry.key) {
                  <div class="flex items-center gap-2 py-0.5">
                    <svg width="22" height="6" class="shrink-0 text-muted-foreground" [style.opacity]="entry.alpha">
                      <line x1="1" y1="3" x2="21" y2="3" stroke="currentColor" stroke-width="2" stroke-linecap="round" [attr.stroke-dasharray]="entry.dash.length ? entry.dash.join(' ') : null" />
                    </svg>
                    <span class="truncate">{{ entry.label }}</span>
                  </div>
                }
              </div>
            }
            <button hlmBtn variant="ghost" size="xs" (click)="showLegend.set(!showLegend())" class="text-xs text-muted-foreground">
              {{ showLegend() ? 'Hide' : 'Legend' }}
            </button>
          </div>

          <div class="absolute bottom-3 left-1/2 -translate-x-1/2 z-20 flex items-center gap-2 bg-popover text-popover-foreground border border-border rounded-lg px-3 py-1.5 text-xs shadow-sm flex-wrap max-w-[90vw]">
            <button hlmBtn variant="outline" size="icon-xs" (click)="zoomBy(1/1.3)" aria-label="Zoom out" class="size-6">−</button>
            <button hlmBtn variant="outline" size="icon-xs" (click)="zoomBy(1.3)" aria-label="Zoom in" class="size-6">+</button>
            <button hlmBtn variant="outline" size="icon-xs" (click)="fitToView()" aria-label="Fit to view" class="size-6">⤢</button>
            <span class="w-px h-4 bg-border mx-1"></span>
            <label class="text-muted-foreground whitespace-nowrap" for="graph-spacing">Spacing</label>
            <input
              id="graph-spacing"
              type="range"
              min="50"
              max="400"
              step="10"
              [value]="linkDistance()"
              (input)="onSpacingChange(+$any($event.target).value)"
              class="w-20 h-1 accent-primary cursor-pointer"
              aria-label="Graph node spacing"
            />
            <span class="text-muted-foreground w-6 text-right tabular-nums">{{ linkDistance() }}</span>
          </div>
        </div>
      </div>
  `,
})
export class GraphViewComponent implements OnInit {
  graphNodes = signal<any[]>([]);
  graphEdges = signal<any[]>([]);
  stats = signal<any | null>(null);
  loading = signal(true);
  error = signal('');
  hoveredNode = signal<HoveredNode | null>(null);
  linkDistance = signal(180);
  showLegend = signal(false);
  pageTypes = signal<PageTypeEntry[]>([]);
  provenanceLegend: ProvenanceEntry[] = [
    { key: 'explicit', label: 'explicit', alpha: 0.75, dash: [] },
    { key: 'derived', label: 'derived', alpha: 0.35, dash: [] },
    { key: 'ambiguous', label: 'ambiguous', alpha: 0.45, dash: [5, 5] },
  ];
  @ViewChild(CanvasGraphDirective, { static: false }) graphDirective?: CanvasGraphDirective;
  private layoutRaf: number | null = null;
  private layoutSim: any = null;
  private spacingTimer: ReturnType<typeof setTimeout> | null = null;

  private router = inject(Router);

  get tooltipX(): number {
    const node = this.hoveredNode();
    if (!node) return 0;
    const x = node.clientX + 16;
    const max = window.innerWidth - 328;
    return Math.max(8, Math.min(x, max));
  }

  get tooltipY(): number {
    const node = this.hoveredNode();
    if (!node) return 0;
    const y = node.clientY - 10;
    const max = window.innerHeight - 120;
    return Math.max(8, Math.min(y, max));
  }

  private buildPageTypes(): PageTypeEntry[] {
    return this.graphColor.allPageTypes();
  }

  constructor(@Inject(ENGINE_PORT) private api: EnginePort, private destroyRef: DestroyRef, private graphColor: GraphColorService) {
    destroyRef.onDestroy(() => {
      if (this.layoutRaf !== null) cancelAnimationFrame(this.layoutRaf);
      if (this.spacingTimer !== null) clearTimeout(this.spacingTimer);
      this.layoutSim?.free();
    });
  }

  ngOnInit() {
    this.pageTypes.set(this.buildPageTypes());
    this.reload();

    this.graphColor.themeChanged$
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe(() => {
        this.pageTypes.set(this.buildPageTypes());
        this.graphDirective?.triggerRender();
      });
  }

  reload() {
    this.loading.set(true);
    this.error.set('');
    this.api.getGraphFull().pipe(takeUntilDestroyed(this.destroyRef)).subscribe({
      next: (res) => {
        if (res.success) {
          this.graphNodes.set(res.nodes || []);
          this.graphEdges.set(res.edges || []);
          this.stats.set(res);
        }
        this.loading.set(false);
        this.graphDirective?.triggerRender();
        this.startLayout();
      },
      error: () => {
        this.error.set('Failed to load graph data');
        this.loading.set(false);
      },
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

  /**
   * The spacing slider drives BOTH the per-link distance and the global
   * many-body repulsion (charge), so it spreads or tightens every node —
   * connected and unconnected alike. Recomputation is debounced so dragging
   * the slider doesn't restart the layout on every input event.
   */
  onSpacingChange(value: number) {
    this.linkDistance.set(value);
    if (this.spacingTimer !== null) clearTimeout(this.spacingTimer);
    this.spacingTimer = setTimeout(() => this.restartLayout(), SPACING_RECOMPUTE_MS);
  }

  private restartLayout() {
    this.spacingTimer = null;
    if (this.graphNodes().length === 0) return;
    if (this.layoutRaf !== null) cancelAnimationFrame(this.layoutRaf);
    this.layoutSim?.free();
    this.layoutSim = null;
    this.layoutRaf = null;
    this.startLayout(true);
  }

  onNodeHover(node: HoveredNode | null) {
    this.hoveredNode.set(node);
  }

  onMouseLeave() {
    this.hoveredNode.set(null);
  }

  /** Run force-directed layout via fjadra WASM in browser */
  private async startLayout(skipFit = false) {
    const nodes = this.graphNodes();
    const nodeCount = nodes.length;
    if (nodeCount === 0) {
      this.loading.set(false);
      return;
    }

    const nodeIndex = new Map(nodes.map((n: any, i: number) => [n.id, i]));
    const sources: number[] = [];
    const targets: number[] = [];
    for (const e of this.graphEdges()) {
      const sId = typeof e.source === 'object' ? e.source.id : e.source;
      const tId = typeof e.target === 'object' ? e.target.id : e.target;
      const s = nodeIndex.get(sId);
      const t = nodeIndex.get(tId);
      if (s !== undefined && t !== undefined) {
        sources.push(s);
        targets.push(t);
      }
    }

    try {
      // @ts-ignore — path resolves at runtime via angular.json assets
      const wasmModule = await import('../../../assets/wasm/fjadra_wasm.js');
      await wasmModule.default();

      const width = window.innerWidth;
      const height = window.innerHeight;
      const centerX = width / 2;
      const centerY = height / 2;
      const spread = Math.min(width, height) * 0.3;
      const linkDistance = this.linkDistance();

      const sim = wasmModule.SimulationHandle.create(
        nodeCount,
        centerX,
        centerY,
        spread,
        new Uint32Array(sources),
        new Uint32Array(targets),
        linkDistance,
        0.3,
        SLIDER_TO_CHARGE(linkDistance),
      );
      this.layoutSim = sim;

      const tickBatch = 15;
      let settled = false;

      const tickLoop = () => {
        for (let i = 0; i < 3; i++) {
          if (sim.is_finished()) {
            settled = true;
            break;
          }
          sim.tick(tickBatch);
        }

        const pos = sim.get_positions();
        const liveNodes = this.graphNodes();
        for (let i = 0; i < nodeCount && i * 2 + 1 < pos.length; i++) {
          liveNodes[i].x = pos[i * 2];
          liveNodes[i].y = pos[i * 2 + 1];
        }
        this.graphDirective?.triggerRender();

        if (!settled) {
          this.layoutRaf = requestAnimationFrame(tickLoop);
        } else {
          this.loading.set(false);
          if (!skipFit) this.fitToView();
          sim.free();
          this.layoutSim = null;
          this.layoutRaf = null;
        }
      };

      this.loading.set(true);
      tickLoop();
    } catch (err: any) {
      this.error.set(err.message || 'Layout computation failed');
      this.loading.set(false);
    }
  }

}
