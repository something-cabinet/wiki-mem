import { Directive, ElementRef, Input, Output, EventEmitter, OnDestroy, AfterViewInit } from '@angular/core';

/** Font used for all edge-label text rendered in the HTML overlay. */
const LABEL_FONT = '11px sans-serif';

/**
 * LOD (level-of-detail) thresholds:
 * - `EDGE_LABEL_MIN_ZOOM`: edge labels are hidden while zoomed out below this
 *   scale, since they overlap and become unreadable noise.
 * - `MAX_EDGE_LABELS`: hard cap on rendered edge labels to bound DOM overlay
 *   cost on very dense graphs (labels are shown for the first N edges).
 * Node labels are not drawn on the canvas (titles surface in the hover
 * tooltip), so there is no node-label LOD threshold here.
 */
const EDGE_LABEL_MIN_ZOOM = 0.4;
const MAX_EDGE_LABELS = 120;

/** Alpha used for the node halo stroke (resolved from the `--ring` token). */
const NODE_HALO_ALPHA = 0.3;

export interface GraphNode {
  id: string;
  title: string;
  page_type: string;
  degree: number;
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
}

export type EdgeProvenance = 'explicit' | 'derived' | 'ambiguous';

export interface GraphEdge {
  source: string | GraphNode;
  target: string | GraphNode;
  edge_type: string;
  provenance?: EdgeProvenance;
}

@Directive({
  selector: 'canvas[wmGraph]',
  standalone: true,
})
export class CanvasGraphDirective implements AfterViewInit, OnDestroy {
  @Input() nodes: GraphNode[] = [];
  @Input() edges: GraphEdge[] = [];
  @Output() nodeClick = new EventEmitter<string>();
  @Output() nodeHover = new EventEmitter<{ id: string; title: string; page_type: string; degree: number; clientX: number; clientY: number } | null>();

  private canvas!: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private transform = { x: 0, y: 0, k: 1 };
  private labelOverlay: HTMLDivElement | null = null;
  private labelElements: HTMLSpanElement[] = [];
  private dpr = 1;

  constructor(private el: ElementRef<HTMLCanvasElement>) {}

  ngAfterViewInit() {
    this.canvas = this.el.nativeElement;
    this.ctx = this.canvas.getContext('2d');
    this.dpr = window.devicePixelRatio || 1;
    this.setupInteraction();
    this.createLabelOverlay();
    this.resizeObserver = new ResizeObserver(() => {
      this.resizeCanvas();
    });
    if (this.canvas.parentElement) {
      this.resizeObserver.observe(this.canvas.parentElement);
    }
    this.resizeCanvas();
  }

  private resizeCanvas() {
    const parent = this.canvas.parentElement;
    if (!parent || !this.ctx) return;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    if (w === 0 || h === 0) return;
    this.canvas.width = w * this.dpr;
    this.canvas.height = h * this.dpr;
    this.canvas.style.width = w + 'px';
    this.canvas.style.height = h + 'px';
    this.ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    this.render();
  }

  private setupInteraction() {
    this.canvas.style.touchAction = 'none';

    let pointerState: {
      id: number;
      node: GraphNode | null;
      startX: number;
      startY: number;
      moved: boolean;
    } | null = null;
    let isPanning = false;
    let panStart = { x: 0, y: 0 };

    this.canvas.addEventListener('pointerdown', (event: PointerEvent) => {
      event.preventDefault();
      this.canvas.setPointerCapture(event.pointerId);
      const [gx, gy] = this.screenToGraph(event.offsetX, event.offsetY);
      const hit = this.hitTest(gx, gy);

      pointerState = {
        id: event.pointerId,
        node: hit,
        startX: event.clientX,
        startY: event.clientY,
        moved: false,
      };

      if (hit) {
        hit.fx = hit.x;
        hit.fy = hit.y;
      } else {
        isPanning = true;
        panStart = { x: event.clientX, y: event.clientY };
      }
    });

    this.canvas.addEventListener('pointermove', (event: PointerEvent) => {
      const [hx, hy] = this.screenToGraph(event.offsetX, event.offsetY);
      const hit = this.hitTest(hx, hy);
      this.canvas.style.cursor = hit ? 'pointer' : 'grab';
      if (hit) {
        this.nodeHover.emit({ id: hit.id, title: hit.title, page_type: hit.page_type, degree: hit.degree, clientX: event.clientX, clientY: event.clientY });
      } else {
        this.nodeHover.emit(null);
      }
    });

    this.canvas.addEventListener('pointermove', (event: PointerEvent) => {
      if (!pointerState) return;

      const dx = event.clientX - pointerState.startX;
      const dy = event.clientY - pointerState.startY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) {
        pointerState.moved = true;
      }

      if (pointerState.node && pointerState.moved) {
        const [gx, gy] = this.screenToGraph(event.offsetX, event.offsetY);
        pointerState.node.fx = gx;
        pointerState.node.fy = gy;
        this.render();
      } else if (isPanning && pointerState.moved) {
        const pdx = event.clientX - panStart.x;
        const pdy = event.clientY - panStart.y;
        this.transform = {
          k: this.transform.k,
          x: this.transform.x + pdx,
          y: this.transform.y + pdy,
        };
        panStart = { x: event.clientX, y: event.clientY };
        this.render();
      }
    });

    this.canvas.addEventListener('pointerup', (event: PointerEvent) => {
      if (!pointerState) return;

      if (pointerState.node && !pointerState.moved) {
        this.nodeClick.emit(pointerState.node.id);
      }

      if (pointerState.node) {
        pointerState.node.x = pointerState.node.fx ?? pointerState.node.x;
        pointerState.node.y = pointerState.node.fy ?? pointerState.node.y;
        this.render();
      }

      pointerState = null;
      isPanning = false;
    });

    this.canvas.addEventListener('pointercancel', () => {
      pointerState = null;
      isPanning = false;
    });

    this.canvas.addEventListener('wheel', (event: WheelEvent) => {
      event.preventDefault();
      const delta = event.deltaY > 0 ? 1 / 1.1 : 1.1;
      const newK = this.transform.k * delta;
      if (newK < 0.01 || newK > 4) return;

      const mx = event.offsetX;
      const my = event.offsetY;
      const newX = mx * (1 - delta) + delta * this.transform.x;
      const newY = my * (1 - delta) + delta * this.transform.y;
      this.transform = { x: newX, y: newY, k: newK };
      this.render();
    }, { passive: false });

    let lastTap = 0;
    this.canvas.addEventListener('pointerdown', (_event: PointerEvent) => {
      const now = Date.now();
      if (now - lastTap < 300 && pointerState?.node) {
        pointerState.node.fx = null as any;
        pointerState.node.fy = null as any;
      }
      lastTap = now;
    });
  }

  /** Convert screen coordinates to graph space (accounts for zoom/pan transform) */
  private screenToGraph(sx: number, sy: number): [number, number] {
    return [
      (sx - this.transform.x) / this.transform.k,
      (sy - this.transform.y) / this.transform.k,
    ];
  }

  /** Compute node radius from degree — uses sqrt scaling for natural distribution */
  private nodeRadius(node: GraphNode): number {
    return Math.max(18, Math.min(55, Math.sqrt(node.degree || 1) * 8 + 10));
  }

  /**
   * Resolve the visual treatment for an edge based on its provenance.
   * - explicit: solid, strongest opacity
   * - derived: solid, lighter opacity
   * - ambiguous: dashed, muted opacity
   * - absent/unknown: neutral solid treatment (matches pre-provenance look)
   */
  private edgeStyle(edge: GraphEdge): { color: string; width: number; dash: number[] } {
    let alpha = 0.6;
    let dash: number[] = [];
    switch (edge.provenance) {
      case 'explicit':
        alpha = 0.75;
        break;
      case 'derived':
        alpha = 0.35;
        break;
      case 'ambiguous':
        alpha = 0.45;
        dash = [5, 5];
        break;
      default:
        alpha = 0.6;
    }
    const color = this.readCssColor(`--edge-type-${edge.edge_type}`, alpha) || `oklch(0.5 0.05 0 / ${alpha})`;
    return {
      color,
      width: 1.5 / this.transform.k,
      dash: dash.map((v) => v / this.transform.k),
    };
  }

  /** Find node at graph coordinates, returns null if none */
  private hitTest(gx: number, gy: number): GraphNode | null {
    for (const node of this.nodes) {
      const nx = node.fx ?? node.x!;
      const ny = node.fy ?? node.y!;
      const dx = gx - nx;
      const dy = gy - ny;
      const r = this.nodeRadius(node);
      if (dx * dx + dy * dy < (r + 8) * (r + 8)) return node;
    }
    return null;
  }

  /** Zoom in/out by a factor (e.g., 1.3 to zoom in, 1/1.3 to zoom out) */
  zoomBy(factor: number) {
    const newK = this.transform.k * factor;
    if (newK < 0.01 || newK > 4) return;
    const cx = this.canvas.clientWidth / 2;
    const cy = this.canvas.clientHeight / 2;
    const newX = cx * (1 - factor) + factor * this.transform.x;
    const newY = cy * (1 - factor) + factor * this.transform.y;
    this.transform = { x: newX, y: newY, k: newK };
    this.render();
  }

  /** Fit graph to viewport */
  fitToView() {
    if (this.nodes.length === 0) return;
    const parent = this.canvas.parentElement!;
    const w = parent.clientWidth;
    const h = parent.clientHeight;
    if (w === 0 || h === 0) return;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const node of this.nodes) {
      const nx = node.fx ?? node.x;
      const ny = node.fy ?? node.y;
      if (nx === undefined || ny === undefined) continue;
      const r = this.nodeRadius(node);
      minX = Math.min(minX, nx - r);
      minY = Math.min(minY, ny - r);
      maxX = Math.max(maxX, nx + r);
      maxY = Math.max(maxY, ny + r);
    }
    if (!isFinite(minX)) return;

    const bbw = maxX - minX;
    const bbh = maxY - minY;
    const padding = 60;
    const maxK = 2;
    const k = Math.min(maxK, Math.min((w - padding * 2) / (bbw || 1), (h - padding * 2) / (bbh || 1)));
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;

    this.transform = { x: w / 2 - k * cx, y: h / 2 - k * cy, k };
    this.render();
  }

  private render() {
    if (!this.ctx) return;
    const ctx = this.ctx;
    const canvas = this.canvas;
    const w = canvas.width / this.dpr;
    const h = canvas.height / this.dpr;

    ctx.save();
    ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    ctx.translate(this.transform.x, this.transform.y);
    ctx.scale(this.transform.k, this.transform.k);

    const edgePairSet = new Set<string>();
    for (const edge of this.edges) {
      const sId = typeof edge.source === 'object' ? edge.source.id : edge.source;
      const tId = typeof edge.target === 'object' ? edge.target.id : edge.target;
      edgePairSet.add(`${sId}→${tId}`);
    }

    for (const edge of this.edges) {
      const source = typeof edge.source === 'object' ? edge.source : this.nodes.find(n => n.id === edge.source);
      const target = typeof edge.target === 'object' ? edge.target : this.nodes.find(n => n.id === edge.target);
      if (!source || !target || source.x === undefined || source.y === undefined || target.x === undefined || target.y === undefined) continue;

      const sId = typeof edge.source === 'object' ? edge.source.id : edge.source;
      const tId = typeof edge.target === 'object' ? edge.target.id : edge.target;
      const hasReverse = edgePairSet.has(`${tId}→${sId}`);

      const dx = target.x - source.x;
      const dy = target.y - source.y;
      const len = Math.sqrt(dx * dx + dy * dy) || 1;
      const nx = -dy / len;

      const style = this.edgeStyle(edge);
      ctx.beginPath();

      if (hasReverse) {
        const offset = sId < tId ? 15 : -15;
        const cpx = (source.x + target.x) / 2 + nx * offset;
        const cpy = (source.y + target.y) / 2 + nx * offset;
        ctx.moveTo(source.x, source.y);
        ctx.quadraticCurveTo(cpx, cpy, target.x, target.y);
      } else {
        ctx.moveTo(source.x, source.y);
        ctx.lineTo(target.x, target.y);
      }

      ctx.strokeStyle = style.color;
      ctx.lineWidth = style.width;
      ctx.setLineDash(style.dash);
      ctx.stroke();
      ctx.setLineDash([]);

      const arrowLen = 8 / this.transform.k;
      const arrowWidth = 4 / this.transform.k;
      const inset = 3 / this.transform.k;

      let angle: number;
      if (hasReverse) {
        const offset = sId < tId ? 15 : -15;
        const cpx = (source.x + target.x) / 2 + nx * offset;
        const cpy = (source.y + target.y) / 2 + nx * offset;
        angle = Math.atan2(target.y - cpy, target.x - cpx);
      } else {
        angle = Math.atan2(dy, dx);
      }

      const tipX = target.x - Math.cos(angle) * inset;
      const tipY = target.y - Math.sin(angle) * inset;
      const baseX = tipX - Math.cos(angle) * arrowLen;
      const baseY = tipY - Math.sin(angle) * arrowLen;
      const perpAngle = angle + Math.PI / 2;
      const blX = baseX + Math.cos(perpAngle) * arrowWidth / 2;
      const blY = baseY + Math.sin(perpAngle) * arrowWidth / 2;
      const brX = baseX - Math.cos(perpAngle) * arrowWidth / 2;
      const brY = baseY - Math.sin(perpAngle) * arrowWidth / 2;

      ctx.beginPath();
      ctx.moveTo(tipX, tipY);
      ctx.lineTo(blX, blY);
      ctx.lineTo(brX, brY);
      ctx.closePath();
      ctx.fillStyle = style.color;
      ctx.fill();
    }

    for (const node of this.nodes) {
      const nx = node.fx ?? node.x;
      const ny = node.fy ?? node.y;
      if (nx === undefined || ny === undefined) continue;
      const radius = this.nodeRadius(node) / this.transform.k;
      const color = this.readCssColor(`--page-type-${node.page_type}`, 0.85) || 'oklch(0.5 0.05 0 / 0.85)';

      ctx.beginPath();
      ctx.arc(nx, ny, radius + 1.5 / this.transform.k, 0, Math.PI * 2);
      ctx.fillStyle = this.readCssColor('--ring', NODE_HALO_ALPHA) || 'oklch(0.5 0.05 0 / 0.3)';
      ctx.fill();

      ctx.beginPath();
      ctx.arc(nx, ny, radius, 0, Math.PI * 2);
      ctx.fillStyle = color;
      ctx.fill();
    }

    ctx.restore();

    this.updateLabelOverlay();
  }

  /** Create HTML overlay for edge labels in WebGL mode */
  private createLabelOverlay(): void {
    this.labelOverlay = document.createElement('div');
    this.labelOverlay.className = 'graph-label-overlay';
    this.labelOverlay.style.cssText = `
      position: absolute; top: 0; left: 0; width: 100%; height: 100%;
      pointer-events: none; overflow: hidden;
    `;
    this.canvas.parentElement?.style.setProperty('position', 'relative');
    this.canvas.insertAdjacentElement('afterend', this.labelOverlay);
  }

  /** Compute edge label positions for the HTML overlay */
  private getEdgeLabels(): { text: string; x: number; y: number; angle: number }[] {
    const labels: { text: string; x: number; y: number; angle: number }[] = [];

    const edgePairSet = new Set<string>();
    for (const edge of this.edges) {
      const sId = typeof edge.source === 'object' ? edge.source.id : edge.source;
      const tId = typeof edge.target === 'object' ? edge.target.id : edge.target;
      edgePairSet.add(`${sId}→${tId}`);
    }

    for (const edge of this.edges) {
      const source = typeof edge.source === 'object' ? edge.source : this.nodes.find(n => n.id === edge.source);
      const target = typeof edge.target === 'object' ? edge.target : this.nodes.find(n => n.id === edge.target);
      if (!source || !target) continue;
      const sx = source.fx ?? source.x;
      const sy = source.fy ?? source.y;
      const tx = target.fx ?? target.x;
      const ty = target.fy ?? target.y;
      if (sx === undefined || sy === undefined || tx === undefined || ty === undefined) continue;

      const sId = typeof edge.source === 'object' ? edge.source.id : edge.source;
      const tId = typeof edge.target === 'object' ? edge.target.id : edge.target;
      const hasReverse = edgePairSet.has(`${tId}→${sId}`);

      const dx = tx - sx;
      const dy = ty - sy;
      const len = Math.sqrt(dx * dx + dy * dy) || 1;
      const nx = -dy / len;

      const labelOffset = hasReverse ? (sId < tId ? 15 : -15) : 0;
      const mx = (sx + tx) / 2 + nx * labelOffset;
      const my = (sy + ty) / 2 + nx * labelOffset;

      let angle = Math.atan2(ty - sy, tx - sx);
      if (angle < -Math.PI / 2 || angle > Math.PI / 2) {
        angle += Math.PI;
      }

      const scx = mx * this.transform.k + this.transform.x;
      const scy = my * this.transform.k + this.transform.y;
      labels.push({ text: edge.edge_type.replace(/_/g, ' '), x: scx, y: scy, angle });
    }
    return labels;
  }

  /** Update edge label positions in the HTML overlay */
  private updateLabelOverlay(): void {
    if (!this.labelOverlay) return;
    const parent = this.canvas.parentElement!;
    const w = parent.clientWidth;
    const h = parent.clientHeight;

    if (this.transform.k < EDGE_LABEL_MIN_ZOOM) {
      for (const el of this.labelElements) el.style.display = 'none';
      return;
    }

    const labels = this.getEdgeLabels().slice(0, MAX_EDGE_LABELS);
    while (this.labelElements.length < labels.length) {
      const el = document.createElement('span');
      el.style.cssText = `
        position: absolute; font: ${LABEL_FONT}; white-space: nowrap;
        pointer-events: none; transform-origin: center center;
        background: color-mix(in oklch, var(--card) 92%, transparent);
        border: 1px solid color-mix(in oklch, var(--border) 55%, transparent);
        padding: 2px 5px; border-radius: 3px;
        color: var(--muted-foreground); line-height: 1;
      `;
      this.labelOverlay.appendChild(el);
      this.labelElements.push(el);
    }
    while (this.labelElements.length > labels.length) {
      const el = this.labelElements.pop()!;
      el.remove();
    }
    for (let i = 0; i < labels.length; i++) {
      const l = labels[i];
      const el = this.labelElements[i];
      el.textContent = l.text;
      el.style.transform = `translate(${l.x}px, ${l.y}px) rotate(${l.angle}rad) translate(-50%, -50%)`;
      if (l.x < -50 || l.x > w + 50 || l.y < -50 || l.y > h + 50) {
        el.style.display = 'none';
      } else {
        el.style.display = '';
      }
    }
  }

  /** Read a CSS custom property value with optional alpha */
  private readCssColor(name: string, alpha?: number): string {
    const color = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    if (alpha !== undefined && color.startsWith('oklch(')) {
      const inner = color.slice(6, -1);
      return `oklch(${inner} / ${alpha})`;
    }
    return color;
  }

  triggerRender(): void {
    this.render();
  }

  ngOnDestroy() {
    this.resizeObserver?.disconnect();
    this.labelOverlay?.remove();
  }
}
