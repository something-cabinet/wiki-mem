import { Directive, ElementRef, Input, Output, EventEmitter, NgZone, OnDestroy, AfterViewInit } from '@angular/core';
import { forceSimulation, forceLink, forceManyBody, forceCenter, forceCollide, SimulationNodeDatum, SimulationLinkDatum } from 'd3-force';

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

  private canvas!: HTMLCanvasElement;
  private ctx!: CanvasRenderingContext2D;
  private sim: d3.Simulation<GraphNode, GraphEdge> | null = null;
  private animFrameId = 0;
  private resizeObserver: ResizeObserver | null = null;

  constructor(private el: ElementRef<HTMLCanvasElement>, private ngZone: NgZone) {}

  ngAfterViewInit() {
    this.canvas = this.el.nativeElement;
    this.ctx = this.canvas.getContext('2d')!;
    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(this.canvas.parentElement!);
    this.resize();
    this.ngZone.runOutsideAngular(() => this.startSimulation());
  }

  private resize() {
    const parent = this.canvas.parentElement!;
    this.canvas.width = parent.clientWidth * devicePixelRatio;
    this.canvas.height = parent.clientHeight * devicePixelRatio;
    this.canvas.style.width = parent.clientWidth + 'px';
    this.canvas.style.height = parent.clientHeight + 'px';
    this.ctx.scale(devicePixelRatio, devicePixelRatio);
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
      this.ctx.beginPath();
      this.ctx.arc(node.x!, node.y!, Math.max(3, Math.min(15, node.degree * 0.5 + 3)), 0, Math.PI * 2);
      this.ctx.fillStyle = this.nodeColor(node.page_type);
      this.ctx.fill();
      this.ctx.strokeStyle = '#fff';
      this.ctx.lineWidth = 1.5;
      this.ctx.stroke();
    }
  }

  private nodeColor(pageType: string): string {
    const colors: Record<string, string> = {
      concept: '#3b82f6', spec: '#22c55e', task: '#f59e0b',
      memory: '#a855f7', pattern: '#ec4899', decision: '#14b8a6',
      howto: '#f97316', reference: '#6b7280',
    };
    return colors[pageType] || '#6b7280';
  }

  onCanvasClick(event: MouseEvent) {
    const rect = this.canvas.getBoundingClientRect();
    const x = event.clientX - rect.left;
    const y = event.clientY - rect.top;
    for (const node of this.nodes) {
      const dx = x - node.x!;
      const dy = y - node.y!;
      if (dx * dx + dy * dy < 100) {
        this.nodeClick.emit(node.id);
        return;
      }
    }
  }

  ngOnDestroy() {
    if (this.sim) this.sim.stop();
    if (this.animFrameId) cancelAnimationFrame(this.animFrameId);
    this.resizeObserver?.disconnect();
  }
}
