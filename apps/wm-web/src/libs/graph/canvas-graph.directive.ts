import { Directive, ElementRef, Input, Output, EventEmitter, NgZone, OnDestroy, AfterViewInit } from '@angular/core';
import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide, SimulationNodeDatum, SimulationLinkDatum, Simulation } from 'd3-force';
import { zoom as d3Zoom, ZoomTransform, zoomIdentity } from 'd3-zoom';

export interface GraphNode extends SimulationNodeDatum {
  id: string;
  title: string;
  page_type: string;
  degree: number;
}

export interface GraphEdge extends SimulationLinkDatum<GraphNode> {
  source: string | GraphNode;
  target: string | GraphNode;
  edge_type: string;
}

@Directive({
  selector: 'canvas[wmGraph]',
  standalone: true,
})
export class CanvasGraphDirective implements AfterViewInit, OnDestroy {
  @Input() nodes: GraphNode[] = [];
  @Input() edges: GraphEdge[] = [];
  @Output() nodeClick = new EventEmitter<string>();
  @Output() nodeHover = new EventEmitter<{ id: string; title: string; page_type: string; degree: number } | null>();

  private canvas!: HTMLCanvasElement;
  private ctx!: CanvasRenderingContext2D;
  private sim: Simulation<GraphNode, GraphEdge> | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private transform = new ZoomTransform(1, 0, 0);
  private draggedNode: GraphNode | null = null;
  private dragOffset = { x: 0, y: 0 };
  private dragActive = false;

  constructor(private el: ElementRef<HTMLCanvasElement>, private ngZone: NgZone) {}

  ngAfterViewInit() {
    this.canvas = this.el.nativeElement;
    this.ctx = this.canvas.getContext('2d')!;
    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(this.canvas.parentElement!);
    this.resize();
    this.ngZone.runOutsideAngular(() => {
      this.setupInteraction();
      this.startSimulation();
    });
  }

  private resize() {
    const parent = this.canvas.parentElement!;
    this.canvas.width = parent.clientWidth * devicePixelRatio;
    this.canvas.height = parent.clientHeight * devicePixelRatio;
    this.canvas.style.width = parent.clientWidth + 'px';
    this.canvas.style.height = parent.clientHeight + 'px';
    this.ctx.scale(devicePixelRatio, devicePixelRatio);
  }

  private setupInteraction() {
    const zoom = d3Zoom<HTMLCanvasElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        this.transform = event.transform;
        if (!this.dragActive) this.render();
      });

    // Apply zoom behavior directly to canvas via native events
    zoom(this.canvas as any, () => this.canvas as any);

    // Mouse interaction handlers
    this.canvas.addEventListener('mousedown', (event: MouseEvent) => {
      const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
      const node = this.hitTest(x, y);
      if (node) {
        this.draggedNode = node;
        this.dragOffset = { x: x - node.x!, y: y - node.y! };
        this.dragActive = true;
        node.fx = node.x;
        node.fy = node.y;
        event.stopPropagation();
      }
    });

    this.canvas.addEventListener('mousemove', (event: MouseEvent) => {
      if (this.draggedNode) {
        const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
        this.draggedNode.fx = x - this.dragOffset.x;
        this.draggedNode.fy = y - this.dragOffset.y;
        this.sim?.alpha(0.3).restart();
      }
      const [hx, hy] = this.screenToGraph(event.offsetX, event.offsetY);
      const hit = this.hitTest(hx, hy);
      this.canvas.style.cursor = hit ? 'pointer' : 'grab';
      if (hit) {
        this.nodeHover.emit({ id: hit.id, title: hit.title, page_type: hit.page_type, degree: hit.degree });
      } else {
        this.nodeHover.emit(null);
      }
    });

    this.canvas.addEventListener('mouseup', () => {
      if (this.draggedNode) {
        this.draggedNode.fx = this.draggedNode.x;
        this.draggedNode.fy = this.draggedNode.y;
        this.draggedNode = null;
        this.dragActive = false;
      }
    });

    this.canvas.addEventListener('dblclick', (event: MouseEvent) => {
      if (this.dragActive) return;
      const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
      const node = this.hitTest(x, y);
      if (node) {
        node.fx = null as any;
        node.fy = null as any;
        this.sim?.alpha(0.3).restart();
      }
    });

    this.canvas.addEventListener('click', (event: MouseEvent) => {
      if (this.dragActive) return;
      const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
      const node = this.hitTest(x, y);
      if (node) this.nodeClick.emit(node.id);
    });
      .on('mousedown.graph', (event: MouseEvent) => {
        const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
        const node = this.hitTest(x, y);
        if (node) {
          this.draggedNode = node;
          this.dragOffset = { x: x - node.x!, y: y - node.y! };
          this.dragActive = true;
          node.fx = node.x;
          node.fy = node.y;
          event.stopPropagation();
        }
      })
      .on('mousemove.graph', (event: MouseEvent) => {
        if (this.draggedNode) {
          const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
          this.draggedNode.fx = x - this.dragOffset.x;
          this.draggedNode.fy = y - this.dragOffset.y;
          this.sim?.alpha(0.3).restart();
        }
        // Hover detection
        const [hx, hy] = this.screenToGraph(event.offsetX, event.offsetY);
        const hit = this.hitTest(hx, hy);
        this.canvas.style.cursor = hit ? 'pointer' : 'grab';
        if (hit) {
          this.nodeHover.emit({ id: hit.id, title: hit.title, page_type: hit.page_type, degree: hit.degree });
        } else {
          this.nodeHover.emit(null);
        }
      })
      .on('mouseup.graph', () => {
        if (this.draggedNode) {
          // Pin the node at its final position
          this.draggedNode.fx = this.draggedNode.x;
          this.draggedNode.fy = this.draggedNode.y;
          this.draggedNode = null;
          this.dragActive = false;
        }
      })
      .on('dblclick.graph', (event: MouseEvent) => {
        if (this.dragActive) return;
        const [x, y] = this.screenToGraph(event.offsetX, event.offsetY);
        const node = this.hitTest(x, y);
        if (node) {
          node.fx = null as any;
          node.fy = null as any;
          this.sim?.alpha(0.3).restart();
        }
      })
      .on('click.graph', (event: MouseEvent) => {
  }

  /** Convert screen coordinates to graph space (accounts for zoom/pan transform) */
  private screenToGraph(sx: number, sy: number): [number, number] {
    return [
      (sx - this.transform.x) / this.transform.k,
      (sy - this.transform.y) / this.transform.k,
    ];
  }

  /** Find node at graph coordinates, returns null if none */
  private hitTest(gx: number, gy: number): GraphNode | null {
    for (const node of this.nodes) {
      const dx = gx - node.x!;
      const dy = gy - node.y!;
      const r = Math.max(3, Math.min(15, (node.degree || 1) * 0.5 + 3));
      if (dx * dx + dy * dy < (r + 3) * (r + 3)) return node;
    }
    return null;
  }

  private startSimulation() {
    if (this.sim) this.sim.stop();
    const w = this.canvas.width / devicePixelRatio;
    const h = this.canvas.height / devicePixelRatio;

    this.sim = forceSimulation<GraphNode>(this.nodes)
      .force('link', forceLink<GraphNode, GraphEdge>(this.edges).id(d => d.id).distance(80).strength(0.3))
      .force('charge', forceManyBody<GraphNode>().strength(-200))
      .force('center', forceCenter(w / 2, h / 2))
      .force('collide', forceCollide<GraphNode>(10))
      .alphaDecay(0.02)
      .velocityDecay(0.3)
      .on('tick', () => this.render());
  }

  private render() {
    const w = this.canvas.width / devicePixelRatio;
    const h = this.canvas.height / devicePixelRatio;
    this.ctx.clearRect(0, 0, w, h);

    // Apply zoom/pan transform
    this.ctx.save();
    this.ctx.translate(this.transform.x, this.transform.y);
    this.ctx.scale(this.transform.k, this.transform.k);

    // Draw edges
    this.ctx.strokeStyle = 'rgba(156, 163, 175, 0.4)';
    this.ctx.lineWidth = 1;
    for (const edge of this.edges) {
      const s = typeof edge.source === 'object' ? edge.source : null;
      const t = typeof edge.target === 'object' ? edge.target : null;
      if (!s || !t) continue;
      this.ctx.beginPath();
      this.ctx.moveTo(s.x!, s.y!);
      this.ctx.lineTo(t.x!, t.y!);
      this.ctx.stroke();
    }

    // Draw nodes
    for (const node of this.nodes) {
      const r = Math.max(3, Math.min(15, (node.degree || 1) * 0.5 + 3));
      this.ctx.beginPath();
      this.ctx.arc(node.x!, node.y!, r, 0, Math.PI * 2);
      this.ctx.fillStyle = this.nodeColor(node.page_type);
      this.ctx.fill();
      this.ctx.strokeStyle = '#fff';
      this.ctx.lineWidth = 1.5;
      this.ctx.stroke();
    }

    this.ctx.restore();
  }

  private nodeColor(pageType: string): string {
    const colors: Record<string, string> = {
      concept: '#3b82f6', spec: '#22c55e', task: '#f59e0b',
      memory: '#a855f7', pattern: '#ec4899', decision: '#14b8a6',
      howto: '#f97316', reference: '#6b7280',
    };
    return colors[pageType] || '#6b7280';
  }

  ngOnDestroy() {
    if (this.sim) this.sim.stop();
    this.resizeObserver?.disconnect();
  }
}
